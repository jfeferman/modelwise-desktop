//! Background work: a check so the icon stays true, and a sync so sessions
//! reach Modelwise without anyone opening the panel.

use std::thread;
use std::time::Duration;

use tauri::AppHandle;

use crate::{commands, settings};

const CHECK_EVERY: Duration = Duration::from_secs(15 * 60);
const SYNC_EVERY_CHECKS: u32 = 4;

pub fn start(app: AppHandle) {
    thread::Builder::new()
        .name("scheduler".into())
        .spawn(move || {
            let mut checks = 0u32;

            loop {
                thread::sleep(CHECK_EVERY);
                checks += 1;

                if checks % SYNC_EVERY_CHECKS == 0 && !settings::paused(&app) {
                    let _ = tauri::async_runtime::block_on(commands::sync());
                }

                let _ = tauri::async_runtime::block_on(commands::check(&app));
            }
        })
        .expect("could not start the scheduler");
}
