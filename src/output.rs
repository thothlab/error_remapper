use crate::matcher::RemapResult;

/// Format the remap result as a JSON string.
pub fn format_result(result: &RemapResult, pretty: bool) -> String {
    if pretty {
        serde_json::to_string_pretty(result).expect("Failed to serialize result")
    } else {
        serde_json::to_string(result).expect("Failed to serialize result")
    }
}
