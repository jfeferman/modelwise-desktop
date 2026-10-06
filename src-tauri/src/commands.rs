//! Everything the panel can ask for. Each is a fixed `modelwise` invocation:
//! the panel never chooses a program or its arguments.

use serde_json::Value;
use tauri::AppHandle;

use crate::{cli, health, tray};

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

            #[cfg(debug_assertions)]
            eprintln!("checked: {} connection(s), worst {:?}", status["connections"].as_array().map_or(0, Vec::len), worst);
            Ok(status)
        }
        Err(message) => {
            tray::show_health(app, health::Health::Broken);
            Err(message)
        }
    }
}
