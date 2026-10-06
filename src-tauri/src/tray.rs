//! The tray icon and the panel under it.
//!
//! The panel is created when the icon is clicked and destroyed when it loses
//! focus, so no web view is alive while nobody is looking.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder, WindowEvent};
use tauri_plugin_positioner::{Position, WindowExt};

use crate::health::Health;

const PANEL: &str = "panel";
const TRAY: &str = "modelwise";

/// When the panel last closed for losing focus. Clicking the icon while the
/// panel is open takes its focus first, and that click must not reopen it.
#[derive(Default)]
pub struct PanelState {
    dismissed: Mutex<Option<Instant>>,
    /// While true, losing focus does not close the panel.
    held: AtomicBool,
}

/// Keeps the panel open through something that takes the focus away, such as
/// approving a sign-in in the browser.
pub fn hold_panel(app: &AppHandle, held: bool) {
    app.state::<PanelState>().held.store(held, Ordering::Relaxed);
}

const REOPEN_GUARD: Duration = Duration::from_millis(300);

pub fn create(app: &AppHandle) -> tauri::Result<()> {
    let sync = MenuItem::with_id(app, "sync", "Sync now", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit Modelwise", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&sync, &quit])?;

    TrayIconBuilder::with_id(TRAY)
        .icon(tauri::include_image!("icons/tray.png"))
        .icon_as_template(true)
        .tooltip("Modelwise")
        .menu(&menu)
        // Left click opens the panel; the menu is on right click.
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "quit" => app.exit(0),
            "sync" => {
                let app = app.clone();
                tauri::async_runtime::spawn(async move {
                    let _ = crate::commands::sync().await;
                    let _ = crate::commands::check(&app).await;
                });
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            let app = tray.app_handle();
            // Tells the positioner where the icon is, to place the panel under it.
            tauri_plugin_positioner::on_tray_event(app, &event);

            if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = event {
                toggle_panel(app);
            }
        })
        .build(app)?;

    Ok(())
}

/// Changes the icon to say how the worst connection is doing.
pub fn show_health(app: &AppHandle, health: Health) {
    let Some(tray) = app.tray_by_id(TRAY) else {
        return;
    };
    let (icon, tooltip) = match health {
        Health::Working => (tauri::include_image!("icons/tray.png"), "Modelwise"),
        Health::Attention => (tauri::include_image!("icons/tray-attention.png"), "Modelwise: needs attention"),
        Health::Broken => (tauri::include_image!("icons/tray-broken.png"), "Modelwise: not working"),
    };

    let _ = tray.set_icon(Some(icon));
    let _ = tray.set_tooltip(Some(tooltip));
}

fn toggle_panel(app: &AppHandle) {
    if let Some(panel) = app.get_webview_window(PANEL) {
        let _ = panel.destroy();
        return;
    }

    let dismissed = *app.state::<PanelState>().dismissed.lock().unwrap();

    if dismissed.is_some_and(|at| at.elapsed() < REOPEN_GUARD) {
        return;
    }

    if let Err(error) = open_panel(app) {
        eprintln!("Could not open the panel: {error}");
    }
}

pub fn open_panel(app: &AppHandle) -> tauri::Result<()> {
    let panel = WebviewWindowBuilder::new(app, PANEL, WebviewUrl::App("index.html".into()))
        .title("Modelwise")
        .inner_size(360.0, 480.0)
        .decorations(false)
        .resizable(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .visible(false)
        .build()?;

    // Fails before the icon has been clicked once, when its place is unknown; the panel then opens where the system puts it.
    let _ = panel.move_window(Position::TrayBottomCenter);
    panel.show()?;
    panel.set_focus()?;

    // When developing, the panel can be held open to look at.
    if crate::held_open() {
        return Ok(());
    }

    let app = app.clone();
    panel.on_window_event(move |event| {
        if let WindowEvent::Focused(false) = event {
            if app.state::<PanelState>().held.load(Ordering::Relaxed) {
                return;
            }

            *app.state::<PanelState>().dismissed.lock().unwrap() = Some(Instant::now());

            if let Some(panel) = app.get_webview_window(PANEL) {
                let _ = panel.destroy();
            }
        }
    });

    Ok(())
}
