//! Everything the panel can ask for. Each is a fixed `modelwise` invocation:
//! the panel never chooses a program, and the only values it supplies are a
//! Modelwise address and a configuration folder the command already listed.

use std::collections::HashSet;
use std::sync::Mutex;

use serde_json::{json, Value};
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_autostart::ManagerExt;

use crate::{cli, health, settings, tray};

/// What the app knows between calls.
#[derive(Default)]
pub struct AppState {
    /// The configuration folders the command listed last, the only ones a
    /// panel request may name.
    known: Mutex<HashSet<String>>,
}

/// Every connection, with its health and what is wrong, and whether each one
/// still works. The tray icon is brought up to date on the way.
#[tauri::command]
pub async fn status(app: AppHandle) -> Result<Value, String> {
    check(&app).await
}

pub async fn check(app: &AppHandle) -> Result<Value, String> {
    let result = tauri::async_runtime::spawn_blocking(|| cli::run(&["status", "--check"]))
        .await
        .map_err(|error| error.to_string())?;

    match result {
        Ok(mut status) => {
            let worst = health::annotate(&mut status);
            tray::show_health(app, worst);
            remember(app, &status);
            status["app"] = json!({
                "paused": settings::paused(app),
                "autostart": app.autolaunch().is_enabled().unwrap_or(false)
            });
            Ok(status)
        }
        Err(message) => {
            tray::show_health(app, health::Health::Broken);
            Err(message)
        }
    }
}

fn remember(app: &AppHandle, status: &Value) {
    let dirs = status["connections"]
        .as_array()
        .map(|connections| {
            connections
                .iter()
                .filter_map(|connection| connection["configDir"].as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();
    *app.state::<AppState>().known.lock().unwrap() = dirs;
}

fn known(app: &AppHandle, config_dir: &str) -> Result<String, String> {
    if app.state::<AppState>().known.lock().unwrap().contains(config_dir) {
        Ok(config_dir.to_string())
    } else {
        Err(format!("{config_dir} is not a connected configuration."))
    }
}

/// Uploads sessions since the last sync, for every connection.
#[tauri::command]
pub async fn sync_now() -> Result<Value, String> {
    sync().await
}

pub async fn sync() -> Result<Value, String> {
    tauri::async_runtime::spawn_blocking(|| cli::run(&["sync"]))
        .await
        .map_err(|error| error.to_string())?
}

/// Writes Claude Code's settings again for one connection.
#[tauri::command]
pub async fn repair(app: AppHandle, config_dir: String) -> Result<Value, String> {
    let dir = known(&app, &config_dir)?;
    tauri::async_runtime::spawn_blocking(move || cli::run(&["repair", "--config-dir", &dir, "--yes"]))
        .await
        .map_err(|error| error.to_string())?
}

/// Removes the settings, the connection and the token for one configuration.
#[tauri::command]
pub async fn disconnect(app: AppHandle, config_dir: String) -> Result<Value, String> {
    let dir = known(&app, &config_dir)?;
    tauri::async_runtime::spawn_blocking(move || cli::run(&["disconnect", "--config-dir", &dir, "--yes"]))
        .await
        .map_err(|error| error.to_string())?
}

/// Signs in to a Modelwise and sets Claude Code's default configuration up.
/// Each step is sent to the panel as a `connect` event; the command opens
/// the browser itself. The panel is held open meanwhile, since approving in
/// the browser takes the focus away.
#[tauri::command]
pub async fn connect(app: AppHandle, url: String) -> Result<(), String> {
    let url = url.trim().to_string();

    if !(url.starts_with("https://") || url.starts_with("http://")) || url.split_whitespace().count() != 1 {
        return Err("Enter the address of your Modelwise, starting with https://".to_string());
    }

    tray::hold_panel(&app, true);
    let emitter = app.clone();
    let result = tauri::async_runtime::spawn_blocking(move || {
        cli::run_streaming(&["connect", "--url", &url, "--yes"], |event| {
            let _ = emitter.emit("connect", event);
        })
    })
    .await
    .map_err(|error| error.to_string());
    tray::hold_panel(&app, false);

    result?
}

/// The person's own usage over the last day, for every connection.
#[tauri::command]
pub async fn usage() -> Result<Value, String> {
    tauri::async_runtime::spawn_blocking(|| cli::run(&["usage", "--period", "24h"]))
        .await
        .map_err(|error| error.to_string())?
}

/// Starts the app at login, or stops doing so.
#[tauri::command]
pub fn set_autostart(app: AppHandle, enabled: bool) -> Result<(), String> {
    let launch = app.autolaunch();
    if enabled { launch.enable() } else { launch.disable() }.map_err(|error| error.to_string())
}

/// Pauses or resumes background sync. Checks carry on either way.
#[tauri::command]
pub fn set_paused(app: AppHandle, paused: bool) -> Result<(), String> {
    settings::set_paused(&app, paused)
}
