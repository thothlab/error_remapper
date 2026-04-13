use crate::matcher::RemapResult;
use crate::settings::OutputSettings;
use serde_json::{Map, Value};
use std::collections::HashMap;

/// Render the output JSON based on the configured template.
///
/// Template placeholders:
///   {{code}}             — remapped error code
///   {{description}}      — remapped description
///   {{matched}}          — whether a match was found
///   {{original_code}}    — original code from input
///   {{original_message}} — original message from input
///   {{input.FIELD}}      — any field from the original input JSON
pub fn format_result(
    result: &RemapResult,
    input_json: &Value,
    settings: &OutputSettings,
) -> String {
    let output = render_template(&settings.template, result, input_json);

    if settings.pretty {
        serde_json::to_string_pretty(&output).expect("Failed to serialize output")
    } else {
        serde_json::to_string(&output).expect("Failed to serialize output")
    }
}

fn render_template(
    template: &HashMap<String, String>,
    result: &RemapResult,
    input_json: &Value,
) -> Value {
    let mut output = Map::new();

    for (key, expr) in template {
        let value = resolve_expression(expr, result, input_json);
        output.insert(key.clone(), value);
    }

    Value::Object(output)
}

fn resolve_expression(expr: &str, result: &RemapResult, input_json: &Value) -> Value {
    let trimmed = expr.trim();

    // Check if the entire expression is a single placeholder
    if trimmed.starts_with("{{") && trimmed.ends_with("}}") && trimmed.matches("{{").count() == 1 {
        let placeholder = trimmed[2..trimmed.len() - 2].trim();
        return resolve_placeholder(placeholder, result, input_json);
    }

    // Otherwise, treat as a string with embedded placeholders
    let mut output = trimmed.to_string();
    // Find all {{...}} patterns and replace them
    loop {
        let start = output.find("{{");
        let end = output.find("}}");
        match (start, end) {
            (Some(s), Some(e)) if e > s => {
                let placeholder = &output[s + 2..e].trim().to_string();
                let value = resolve_placeholder(placeholder, result, input_json);
                let replacement = match &value {
                    Value::String(s) => s.clone(),
                    Value::Bool(b) => b.to_string(),
                    Value::Number(n) => n.to_string(),
                    Value::Null => "null".to_string(),
                    other => other.to_string(),
                };
                output = format!("{}{}{}", &output[..s], replacement, &output[e + 2..]);
            }
            _ => break,
        }
    }

    Value::String(output)
}

fn resolve_placeholder(placeholder: &str, result: &RemapResult, input_json: &Value) -> Value {
    match placeholder {
        "code" => Value::String(result.code.clone()),
        "description" => Value::String(result.custom_desc.clone()),
        "matched" => Value::Bool(result.matched),
        "original_code" => Value::String(result.original_code.clone()),
        "original_message" => Value::String(result.original_message.clone()),
        other if other.starts_with("input.") => {
            let field_name = &other[6..];
            extract_input_field(input_json, field_name)
        }
        _ => Value::Null,
    }
}

/// Extract a field from input JSON by name (supports nested access with dots)
fn extract_input_field(value: &Value, field_path: &str) -> Value {
    let parts: Vec<&str> = field_path.splitn(2, '.').collect();
    match value {
        Value::Object(map) => {
            if let Some(v) = map.get(parts[0]) {
                if parts.len() == 1 {
                    v.clone()
                } else {
                    extract_input_field(v, parts[1])
                }
            } else {
                Value::Null
            }
        }
        _ => Value::Null,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::matcher::RemapResult;

    fn make_result() -> RemapResult {
        RemapResult {
            code: "81005".into(),
            custom_desc: "Перевод отклонён банком получателя".into(),
            matched: true,
            original_code: "3011".into(),
            original_message: "Не пройден фрод".into(),
        }
    }

    fn make_input() -> Value {
        serde_json::json!({
            "statusCode": "3011",
            "errorText": "Не пройден фрод",
            "ErrorDescription": "Процесс не был пройден через антифрод",
            "nested": {
                "detail": "some detail"
            }
        })
    }

    #[test]
    fn test_simple_placeholders() {
        let mut template = HashMap::new();
        template.insert("statusCode".into(), "{{code}}".into());
        template.insert("errorText".into(), "{{description}}".into());

        let settings = OutputSettings {
            template,
            pretty: false,
        };

        let output = format_result(&make_result(), &make_input(), &settings);
        let parsed: Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed["statusCode"], "81005");
        assert_eq!(parsed["errorText"], "Перевод отклонён банком получателя");
    }

    #[test]
    fn test_input_passthrough() {
        let mut template = HashMap::new();
        template.insert("statusCode".into(), "{{code}}".into());
        template.insert("errorText".into(), "{{description}}".into());
        template.insert("ErrorDescription".into(), "{{input.ErrorDescription}}".into());

        let settings = OutputSettings {
            template,
            pretty: false,
        };

        let output = format_result(&make_result(), &make_input(), &settings);
        let parsed: Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed["statusCode"], "81005");
        assert_eq!(
            parsed["ErrorDescription"],
            "Процесс не был пройден через антифрод"
        );
    }

    #[test]
    fn test_nested_input_field() {
        let mut template = HashMap::new();
        template.insert("detail".into(), "{{input.nested.detail}}".into());

        let settings = OutputSettings {
            template,
            pretty: false,
        };

        let output = format_result(&make_result(), &make_input(), &settings);
        let parsed: Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed["detail"], "some detail");
    }

    #[test]
    fn test_matched_bool() {
        let mut template = HashMap::new();
        template.insert("ok".into(), "{{matched}}".into());

        let settings = OutputSettings {
            template,
            pretty: false,
        };

        let output = format_result(&make_result(), &make_input(), &settings);
        let parsed: Value = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed["ok"], true);
    }
}
