//! Settings persistence

use crate::types::AppSettings;
use std::fs;
use std::path::PathBuf;

/// Get the settings file path
pub fn settings_path() -> Option<PathBuf> {
    dirs::config_dir().map(|p| p.join("WindowLasso").join("settings.json"))
}

/// Load settings from disk
pub fn load_settings() -> AppSettings {
    let path = match settings_path() {
        Some(p) => p,
        None => return AppSettings::default(),
    };

    match fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
        Err(_) => AppSettings::default(),
    }
}

/// Save settings to disk
pub fn save_settings(settings: &AppSettings) -> Result<(), String> {
    let path = settings_path().ok_or("Could not determine config directory")?;

    // Ensure directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;
    }

    let json = serde_json::to_string_pretty(settings)
        .map_err(|e| format!("Failed to serialize settings: {}", e))?;

    fs::write(&path, json).map_err(|e| format!("Failed to write settings: {}", e))?;

    Ok(())
}
