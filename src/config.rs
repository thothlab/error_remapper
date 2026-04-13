use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Single error entry from the YAML vocabulary
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorEntry {
    /// Error code from the source system (used for exact matching)
    pub key: String,
    /// Substring pattern for fuzzy matching against error text
    pub substring: String,
    /// New error code to return
    pub code: String,
    /// Optional custom description to replace the original error text
    pub custom_desc: Option<String>,
}

/// The vocabulary section of the YAML config
#[derive(Debug, Deserialize)]
pub struct ErrorSection {
    pub vocabulary: Vec<ErrorEntry>,
}

/// Root YAML structure (top-level key maps to ErrorSection)
pub type ErrorConfig = HashMap<String, ErrorSection>;

/// Load and parse the YAML error dictionary
pub fn load_error_config(path: &Path) -> Result<Vec<ErrorEntry>, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read errors YAML '{}': {}", path.display(), e))?;

    let config: ErrorConfig = serde_yaml::from_str(&content)
        .map_err(|e| format!("Failed to parse errors YAML '{}': {}", path.display(), e))?;

    let mut entries = Vec::new();
    for (_section_name, section) in config {
        entries.extend(section.vocabulary);
    }

    if entries.is_empty() {
        return Err(format!("No error entries found in '{}'", path.display()));
    }

    log::info!("Loaded {} error entries from '{}'", entries.len(), path.display());
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_temp_yaml(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(content.as_bytes()).unwrap();
        file
    }

    #[test]
    fn test_load_valid_config() {
        let yaml = r#"
preprocess-error:
  vocabulary:
    - key: "2001"
      substring: "unexpected symbol:"
      customDesc: "Custom description"
      code: "81002"
    - key: "2002"
      substring: "Уточните у получателя"
      code: "81001"
"#;
        let file = create_temp_yaml(yaml);
        let entries = load_error_config(file.path()).unwrap();
        assert_eq!(entries.len(), 2);

        let entry_2001 = entries.iter().find(|e| e.key == "2001").unwrap();
        assert_eq!(entry_2001.code, "81002");
        assert_eq!(entry_2001.custom_desc.as_deref(), Some("Custom description"));

        let entry_2002 = entries.iter().find(|e| e.key == "2002").unwrap();
        assert_eq!(entry_2002.code, "81001");
        assert!(entry_2002.custom_desc.is_none());
    }

    #[test]
    fn test_load_empty_vocabulary() {
        let yaml = r#"
preprocess-error:
  vocabulary: []
"#;
        let file = create_temp_yaml(yaml);
        let result = load_error_config(file.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_load_nonexistent_file() {
        let result = load_error_config(Path::new("/nonexistent/file.yaml"));
        assert!(result.is_err());
    }
}
