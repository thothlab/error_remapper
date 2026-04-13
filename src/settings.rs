use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub input: InputSettings,
    pub matching: MatchingSettings,
    pub output: OutputSettings,
    pub files: FileSettings,
}

#[derive(Debug, Deserialize)]
pub struct InputSettings {
    pub code_fields: Vec<String>,
    pub message_fields: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct MatchingSettings {
    pub fuzzy_threshold: f64,
}

#[derive(Debug, Deserialize)]
pub struct OutputSettings {
    /// Template mapping: output field name → source expression.
    ///
    /// Available placeholders:
    ///   {{code}}         — remapped error code
    ///   {{description}}  — remapped description
    ///   {{matched}}      — whether a match was found (true/false)
    ///   {{input.FIELD}}  — any field from the original input JSON
    ///   {{original_code}}    — original code from input
    ///   {{original_message}} — original message from input
    pub template: HashMap<String, String>,

    /// Pretty-print the output JSON
    #[serde(default)]
    pub pretty: bool,
}

#[derive(Debug, Deserialize)]
pub struct FileSettings {
    pub errors_yaml: String,
}

impl Default for Settings {
    fn default() -> Self {
        let mut template = HashMap::new();
        template.insert("code".into(), "{{code}}".into());
        template.insert("customDesc".into(), "{{description}}".into());
        template.insert("matched".into(), "{{matched}}".into());

        Settings {
            input: InputSettings {
                code_fields: vec!["code".into(), "errorCode".into(), "statusCode".into()],
                message_fields: vec![
                    "title".into(),
                    "message".into(),
                    "errorMessage".into(),
                    "errorText".into(),
                ],
            },
            matching: MatchingSettings {
                fuzzy_threshold: 0.4,
            },
            output: OutputSettings {
                template,
                pretty: false,
            },
            files: FileSettings {
                errors_yaml: "config/errors.yaml".into(),
            },
        }
    }
}

pub fn load_settings(path: &Path) -> Result<Settings, String> {
    if !path.exists() {
        log::warn!(
            "Settings file '{}' not found, using defaults",
            path.display()
        );
        return Ok(Settings::default());
    }

    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read settings '{}': {}", path.display(), e))?;

    let settings: Settings = toml::from_str(&content)
        .map_err(|e| format!("Failed to parse settings '{}': {}", path.display(), e))?;

    log::info!("Loaded settings from '{}'", path.display());
    Ok(settings)
}
