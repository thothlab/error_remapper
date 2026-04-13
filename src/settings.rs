use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub input: InputSettings,
    pub matching: MatchingSettings,
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
pub struct FileSettings {
    pub errors_yaml: String,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            input: InputSettings {
                code_fields: vec!["code".into(), "errorCode".into()],
                message_fields: vec!["title".into(), "message".into(), "errorMessage".into()],
            },
            matching: MatchingSettings {
                fuzzy_threshold: 0.4,
            },
            files: FileSettings {
                errors_yaml: "config/errors.yaml".into(),
            },
        }
    }
}

pub fn load_settings(path: &Path) -> Result<Settings, String> {
    if !path.exists() {
        log::warn!("Settings file '{}' not found, using defaults", path.display());
        return Ok(Settings::default());
    }

    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read settings '{}': {}", path.display(), e))?;

    let settings: Settings = toml::from_str(&content)
        .map_err(|e| format!("Failed to parse settings '{}': {}", path.display(), e))?;

    log::info!("Loaded settings from '{}'", path.display());
    Ok(settings)
}
