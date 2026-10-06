//! The app's own settings: so far, whether background sync is paused. Kept
//! in the app's configuration folder, apart from the command's files.

use std::fs;
use std::path::PathBuf;

use serde_json::{json, Value};
use tauri::{AppHandle, Manager};

fn file(app: &AppHandle) -> Option<PathBuf> {
    app.path().app_config_dir().ok().map(|dir| dir.join("settings.json"))
}

pub fn paused(app: &AppHandle) -> bool {
    file(app)
        .and_then(|path| fs::read_to_string(path).ok())
        .and_then(|raw| serde_json::from_str::<Value>(&raw).ok())
        .and_then(|value| value.get("paused").and_then(Value::as_bool))
        .unwrap_or(false)
}

pub fn set_paused(app: &AppHandle, paused: bool) -> Result<(), String> {
    let path = file(app).ok_or("No configuration folder for this app")?;

    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).map_err(|error| error.to_string())?;
    }

    fs::write(&path, format!("{}\n", json!({ "paused": paused }))).map_err(|error| error.to_string())
}
