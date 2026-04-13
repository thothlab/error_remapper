pub mod config;
pub mod input;
pub mod matcher;
pub mod output;
pub mod settings;

use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::path::Path;

/// Core remap function usable from Rust code directly.
///
/// `input_json` — JSON string with the error
/// `config_dir` — path to directory containing settings.toml and errors.yaml
///
/// Returns the output JSON string.
pub fn remap(input_json: &str, config_dir: &str) -> Result<String, String> {
    let settings_path = Path::new(config_dir).join("settings.toml");
    let settings = settings::load_settings(&settings_path)?;

    let errors_path = Path::new(config_dir).join(&settings.files.errors_yaml);
    // errors_yaml may be relative to config_dir or absolute
    let errors_path = if errors_path.exists() {
        errors_path
    } else {
        // Try as-is (e.g. "config/errors.yaml" relative to cwd)
        std::path::PathBuf::from(&settings.files.errors_yaml)
    };

    let entries = config::load_error_config(&errors_path)?;

    let input_value: serde_json::Value = serde_json::from_str(input_json)
        .map_err(|e| format!("Failed to parse input JSON: {}", e))?;

    let parsed = input::parse_error_json(
        input_json,
        &settings.input.code_fields,
        &settings.input.message_fields,
    )?;

    let result = matcher::find_match(&parsed, &entries, settings.matching.fuzzy_threshold);

    let json_output = output::format_result(&result, &input_value, &settings.output);
    Ok(json_output)
}

// ============================================================
// C-compatible API for JNA / JNI
// ============================================================

/// Remap an error JSON string using the given config directory.
///
/// # Arguments
/// * `input_json` — C string with the input JSON
/// * `config_dir` — C string with the path to config directory
///
/// # Returns
/// A newly allocated C string with the result JSON.
/// The caller MUST free it by calling `error_remapper_free`.
/// On error, returns a JSON string with an "error" field.
#[no_mangle]
pub extern "C" fn error_remapper_remap(
    input_json: *const c_char,
    config_dir: *const c_char,
) -> *mut c_char {
    let input = match unsafe { CStr::from_ptr(input_json) }.to_str() {
        Ok(s) => s,
        Err(e) => return error_to_cstring(&format!("Invalid UTF-8 in input_json: {}", e)),
    };

    let config = match unsafe { CStr::from_ptr(config_dir) }.to_str() {
        Ok(s) => s,
        Err(e) => return error_to_cstring(&format!("Invalid UTF-8 in config_dir: {}", e)),
    };

    match remap(input, config) {
        Ok(result) => CString::new(result).unwrap_or_default().into_raw(),
        Err(e) => error_to_cstring(&e),
    }
}

/// Free a string returned by `error_remapper_remap`.
/// Must be called exactly once for each non-null return value.
#[no_mangle]
pub extern "C" fn error_remapper_free(s: *mut c_char) {
    if !s.is_null() {
        unsafe {
            let _ = CString::from_raw(s);
        }
    }
}

fn error_to_cstring(msg: &str) -> *mut c_char {
    let json = serde_json::json!({"error": msg}).to_string();
    CString::new(json).unwrap_or_default().into_raw()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remap_function() {
        let input = r#"{"statusCode": "3011", "errorText": "Не пройден фрод", "ErrorDescription": "Тест"}"#;
        let result = remap(input, "config").unwrap();
        assert!(result.contains("81005"));
        assert!(result.contains("Перевод отклонён банком получателя"));
    }

    #[test]
    fn test_remap_invalid_json() {
        let result = remap("not json", "config");
        assert!(result.is_err());
    }
}
