use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Persisted app settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub indexed_folders: Vec<String>,
    pub hotkey: String,
    pub autostart: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            indexed_folders: default_indexed_folders(),
            hotkey: "Alt+Space".into(),
            autostart: false,
        }
    }
}

/// Default user folders to index: Documents, Downloads, Desktop, Pictures, Videos.
pub fn default_indexed_folders() -> Vec<String> {
    let mut list = Vec::new();
    if let Some(user) = directories::UserDirs::new() {
        if let Some(p) = user.document_dir() {
            list.push(p.to_string_lossy().into_owned());
        }
        if let Some(p) = user.download_dir() {
            list.push(p.to_string_lossy().into_owned());
        }
        if let Some(p) = user.desktop_dir() {
            list.push(p.to_string_lossy().into_owned());
        }
        if let Some(p) = user.picture_dir() {
            list.push(p.to_string_lossy().into_owned());
        }
        if let Some(p) = user.video_dir() {
            list.push(p.to_string_lossy().into_owned());
        }
    }
    list
}

fn settings_path() -> PathBuf {
    directories::ProjectDirs::from("com", "cariyuk", "app")
        .map(|dirs| dirs.data_dir().to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."))
        .join("settings.json")
}

pub fn load() -> AppSettings {
    let path = settings_path();
    std::fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save(settings: &AppSettings) -> Result<(), Box<dyn std::error::Error>> {
    let path = settings_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let json = serde_json::to_string_pretty(settings)?;
    std::fs::write(path, json)?;
    Ok(())
}
