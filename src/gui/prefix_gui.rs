use std::env;
use std::path::PathBuf;
use rfd::FileDialog;
use slint::{ComponentHandle, SharedString};
use which::which;
use crate::compatibility_tools::compat_tools_wrapper::{get_compat_tool_from_config, reset_prefix, run_in_prefix, run_wiretricks_in_prefix};
use crate::config::config_loader::get_steam_app_id;
use crate::gui::dialog::show_message_dialog;
use crate::PrefixSettingsGUI;

pub fn show_gui() {
    let window = PrefixSettingsGUI::new().unwrap();
    let steam_app_id = get_steam_app_id().unwrap_or("".to_string());
    window.set_game_app_id(SharedString::from(&steam_app_id));

    let data_path = PathBuf::from(env::var("STEAM_COMPAT_DATA_PATH").expect("STEAM_COMPAT_DATA_PATH must be set"));
    window.set_prefix_path(SharedString::from(data_path.to_str().unwrap_or("")));

    let compat_tool = get_compat_tool_from_config();
    window.set_runner_name(SharedString::from(compat_tool.name));

    window.on_run_in_prefix(|in_terminal| {
            if let Some(path) = FileDialog::new()
                .add_filter("Windows Executables", &["exe","msi","msix"])
                .add_filter("Windows Scripts", &["bat", "cmd"])
                .pick_file() {
                tokio::spawn(async move {
                    let status = run_in_prefix(path, in_terminal).await;
                    show_message_dialog(&format!("App finished with exit code {}", status.unwrap()));
                });
            }
        });

    window.on_recreate_prefix({
        let weak_window = window.as_weak();
        move || {
            let weak_window = weak_window.clone();
            tokio::spawn(async {
                let reset = reset_prefix().await;
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
        move || {
            let weak_window = weak_window.clone();
            tokio::spawn(async move {
                if which("winetricks").is_err() {
                    show_message_dialog("Could not find winetricks in the system path.");
                } else {
                    let run_winetricks = run_wiretricks_in_prefix().await;
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
