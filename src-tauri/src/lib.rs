//! The container: a tray icon, a panel that exists only while it is open, and
//! the embedded `modelwise` command, which does all the work.
//!
//! Nothing here signs in, edits Claude Code's settings or uploads anything.
//! That is the command's job (`@modelwise/cli`), so that the terminal and the
//! menu bar cannot disagree about what happens.

mod cli;
mod commands;
mod health;
mod scheduler;
mod settings;
mod tray;

/// In a debug build, MODELWISE_DESKTOP_OPEN opens the panel at launch and keeps
/// it open, for working on it without clicking the tray each time.
pub(crate) fn held_open() -> bool {
    cfg!(debug_assertions) && std::env::var_os("MODELWISE_DESKTOP_OPEN").is_some()
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_positioner::init())
        .plugin(tauri_plugin_autostart::init(tauri_plugin_autostart::MacosLauncher::LaunchAgent, None))
        .manage(tray::PanelState::default())
        .manage(commands::AppState::default())
        .invoke_handler(tauri::generate_handler![
            commands::status,
            commands::sync_now,
            commands::repair,
            commands::disconnect,
            commands::connect,
            commands::usage,
            commands::set_paused,
            commands::set_autostart
        ])
        .setup(|app| {
            // A menu bar app: no Dock icon, no app menu.
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            tray::create(app.handle())?;

            // So the icon shows the state before anyone opens the panel.
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                let _ = commands::check(&handle).await;
            });
            scheduler::start(app.handle().clone());

            if held_open() {
                tray::open_panel(app.handle())?;
            } else if cfg!(debug_assertions) && std::env::var_os("MODELWISE_DESKTOP_CLICK").is_some() {
                // As if the icon were clicked, blur-to-close and all.
                tray::toggle_panel(app.handle());
            }

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("could not start Modelwise")
        .run(|_app, event| {
            // Closing the panel closes the last window, which is not quitting.
            if let tauri::RunEvent::ExitRequested { api, code, .. } = event {
                if code.is_none() {
                    api.prevent_exit();
                }
            }
        });
}
