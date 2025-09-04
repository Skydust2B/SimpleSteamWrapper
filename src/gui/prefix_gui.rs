use std::process::Command;
use std::sync::Arc;
use tokio::sync::{Mutex};
use rfd::FileDialog;
use slint::{ComponentHandle, SharedString};
use which::which;
use crate::compatibility_tools::compat_tool::{get_compat_tool_from_config};
use crate::compatibility_tools::app_prefix::{AppPrefix};
use crate::config::config_loader::{get_steam_app_id};
use crate::gui::dialog::show_message_dialog;
use crate::PrefixSettingsGUI;

pub fn show_gui() {
    let window = PrefixSettingsGUI::new().unwrap();
    let steam_app_id = get_steam_app_id().unwrap_or("".to_string());
    window.set_game_app_id(SharedString::from(&steam_app_id));

    let shared_pfx_ref: Arc<Mutex<AppPrefix>> = Arc::new(Mutex::new(AppPrefix::from_env()));

    window.set_prefix_path({
        let prefix = AppPrefix::from_env();
        SharedString::from(prefix.as_path().to_str().unwrap_or(""))
    });

    let compat_tool = get_compat_tool_from_config();
    window.set_runner_name(SharedString::from(compat_tool.name));

    window.on_run_in_prefix({
        let shared_pfx_ref = Arc::clone(&shared_pfx_ref);
        move |in_terminal| {
            if let Some(path) = FileDialog::new()
                .add_filter("Windows Executables", &["exe","msi","msix"])
                .add_filter("Windows Scripts", &["bat", "cmd"])
                .pick_file() {
                let shared_pfx_ref = Arc::clone(&shared_pfx_ref);
                tokio::spawn(async move {
                    let borrowed_pfx_ref = shared_pfx_ref.lock().await;
                    let status = borrowed_pfx_ref.run_in_prefix(
                        &get_compat_tool_from_config(),
                        Command::new(path),
                        in_terminal
                    ).await;
                    show_message_dialog(&format!("App finished with exit code {}", status.unwrap()));
                });
            }
        }
    });

    window.on_recreate_prefix({
        let weak_window = window.as_weak();
        let shared_pfx_ref = Arc::clone(&shared_pfx_ref);
        move || {
            let weak_window = weak_window.clone();
            let shared_pfx_ref = Arc::clone(&shared_pfx_ref);
            tokio::spawn(async move {
                let borrowed_pfx_ref = shared_pfx_ref.lock().await;
                let reset = borrowed_pfx_ref.reset_prefix(&get_compat_tool_from_config()).await;
                if reset.is_ok() {
                    show_message_dialog("Successfully recreated prefix");
                } else {
                    show_message_dialog("Failed to recreated prefix");
                }
                slint::invoke_from_event_loop(move || {
                    let upgraded_win = weak_window.upgrade().unwrap();
                    upgraded_win.set_recreating_prefix(false);
                })
            });
        }
    });

    window.on_run_winetricks({
        let weak_window = window.as_weak().clone();
        let shared_pfx_ref = Arc::clone(&shared_pfx_ref);
        move || {
            let weak_window = weak_window.clone();
            let shared_pfx_ref = Arc::clone(&shared_pfx_ref);
            tokio::spawn(async move {
                if which("winetricks").is_err() {
                    show_message_dialog("Could not find winetricks in the system path.");
                } else {
                    let borrowed_pfx_ref = shared_pfx_ref.lock().await;
                    let run_winetricks = borrowed_pfx_ref.run_wiretricks(&get_compat_tool_from_config()).await;
                    if run_winetricks.is_err() {
                        show_message_dialog("Winetricks failed to run");
                    }
                }
                slint::invoke_from_event_loop(move || {
                    let upgraded_win = weak_window.upgrade().unwrap();
                    upgraded_win.set_running_winetricks(false);
                })
            });
        }
    });
    
    let _ = window.show();
}
