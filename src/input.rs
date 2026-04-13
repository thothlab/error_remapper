use serde_json::Value;

/// Extracted error fields from input JSON
#[derive(Debug)]
pub struct ParsedError {
    /// Error code (as string)
    pub code: Option<String>,
    /// Error message text
    pub message: Option<String>,
}

/// Recursively search a JSON value for fields matching the given names.
/// Returns the first match found (depth-first).
fn find_field(value: &Value, field_names: &[String]) -> Option<String> {
    match value {
        Value::Object(map) => {
            // First check direct children
            for name in field_names {
                if let Some(v) = map.get(name) {
                    return match v {
                        Value::String(s) => Some(s.clone()),
                        Value::Number(n) => Some(n.to_string()),
                        _ => Some(v.to_string()),
                    };
                }
            }
            // Then recurse into nested objects
            for (_key, v) in map {
                if let Some(result) = find_field(v, field_names) {
                    return Some(result);
                }
            }
            None
        }
        Value::Array(arr) => {
            for item in arr {
                if let Some(result) = find_field(item, field_names) {
                    return Some(result);
                }
            }
            None
        }
        _ => None,
    }
}

/// Parse input JSON string and extract error code and message
/// using the configured field names.
pub fn parse_error_json(
    json_str: &str,
    code_fields: &[String],
    message_fields: &[String],
) -> Result<ParsedError, String> {
    let value: Value = serde_json::from_str(json_str)
        .map_err(|e| format!("Failed to parse input JSON: {}", e))?;

    let code = find_field(&value, code_fields);
    let message = find_field(&value, message_fields);

    log::info!("Parsed error - code: {:?}, message: {:?}", code, message);

    Ok(ParsedError { code, message })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn code_fields() -> Vec<String> {
        vec!["code".into(), "errorCode".into()]
    }

    fn message_fields() -> Vec<String> {
        vec!["title".into(), "message".into(), "errorMessage".into()]
    }

    #[test]
    fn test_nested_error_object() {
        let json = r#"{"error":{"code":-2,"title":"Server unavailable"}}"#;
        let parsed = parse_error_json(json, &code_fields(), &message_fields()).unwrap();
        assert_eq!(parsed.code.as_deref(), Some("-2"));
        assert_eq!(parsed.message.as_deref(), Some("Server unavailable"));
    }

    #[test]
    fn test_flat_error_object() {
        let json = r#"{"errorCode":"3011","message":"Не пройден фрод-мониторинг"}"#;
        let parsed = parse_error_json(json, &code_fields(), &message_fields()).unwrap();
        assert_eq!(parsed.code.as_deref(), Some("3011"));
        assert_eq!(parsed.message.as_deref(), Some("Не пройден фрод-мониторинг"));
    }

    #[test]
    fn test_missing_fields() {
        let json = r#"{"status":"error","description":"Something went wrong"}"#;
        let parsed = parse_error_json(json, &code_fields(), &message_fields()).unwrap();
        assert!(parsed.code.is_none());
        assert!(parsed.message.is_none());
    }

    #[test]
    fn test_invalid_json() {
        let result = parse_error_json("not json", &code_fields(), &message_fields());
        assert!(result.is_err());
    }
}
