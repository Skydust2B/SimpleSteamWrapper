use std::process::Command;
use std::sync::Arc;
use tokio::sync::{Mutex};
use rfd::FileDialog;
use slint::{ComponentHandle, SharedString};
use which::which;
use crate::utils::command_utils::parse_cmdline;
use crate::compatibility_tools::compat_tool::{get_compat_tool_from_config};
use crate::compatibility_tools::app_prefix::{AppPrefix};
use crate::gui::dialog::show_message_dialog;
use crate::PrefixSettingsGUI;
use crate::steam::steam::get_steam_env_app_id;

pub fn show_gui() {
    let window = PrefixSettingsGUI::new().unwrap();
    let steam_app_id = get_steam_env_app_id().unwrap_or("".to_string());
    window.set_game_app_id(steam_app_id.into());

    let shared_pfx_ref: Arc<Mutex<AppPrefix>> = Arc::new(Mutex::new(AppPrefix::from_env()));

    window.set_prefix_path({
        let prefix = AppPrefix::from_env();
        prefix.as_path().to_str().unwrap_or("").into()
    });

    if let Some(cfg_compat_tool) = get_compat_tool_from_config() {
        window.set_runner_name(cfg_compat_tool.name.into());
    }

    window.on_run_in_prefix({
        let shared_pfx_ref = Arc::clone(&shared_pfx_ref);
        move |in_terminal, custom_cmd| {
            let cfg_compat_tool = get_compat_tool_from_config();
            if cfg_compat_tool.is_none() {
                show_message_dialog("No compat tool selected.");
                return;
            }

            let new_command = if custom_cmd.is_empty() {
                FileDialog::new()
                    .add_filter("Windows Executables", &["exe","msi","msix"])
                    .add_filter("Windows Scripts", &["bat", "cmd"])
                    .pick_file().and_then(|p| Some(SharedString::from(p.to_str().unwrap_or_default())))
            } else {
                Some(custom_cmd)
            };

            if let Some(cmd) = new_command {
                let shared_pfx_ref = shared_pfx_ref.clone();
                let parsed_cmd = parse_cmdline(cmd.as_str());
                tokio::spawn(async move {
                    let borrowed_pfx_ref = shared_pfx_ref.lock().await;
                    let mut cmd_to_run = Command::new(parsed_cmd.progname);
                    cmd_to_run.envs(parsed_cmd.env);
                    cmd_to_run.args(parsed_cmd.args);

                    let status = borrowed_pfx_ref.run_in_prefix(
                        &cfg_compat_tool.unwrap(),
                        cmd_to_run,
                        in_terminal
                    ).await;
                    show_message_dialog(&format!("App finished with exit code {}", status.unwrap()));
                });
            }
        }
    });

    window.on_recreate_prefix({
        let weak_window = window.as_weak();
        let shared_pfx_ref = shared_pfx_ref.clone();
        move || {
            let weak_window = weak_window.clone();
            let shared_pfx_ref = shared_pfx_ref.clone();

            let cfg_compat_tool = get_compat_tool_from_config();
            if cfg_compat_tool.is_none() {
                show_message_dialog("No compat tool selected.");
                return;
            }

            tokio::spawn(async move {
                let borrowed_pfx_ref = shared_pfx_ref.lock().await;
                let reset = borrowed_pfx_ref.reset_prefix(&cfg_compat_tool.unwrap()).await;
                if reset.is_ok() {
                    show_message_dialog("Successfully recreated prefix");
                } else {
                    show_message_dialog("Failed to recreated prefix");
                }
                let _ = weak_window.upgrade_in_event_loop(|window| {
                    window.set_recreating_prefix(false);
                });
            });
        }
    });

    window.on_run_winetricks({
        let weak_window = window.as_weak().clone();
        let shared_pfx_ref = shared_pfx_ref.clone();
        move || {
            let weak_window = weak_window.clone();
            let shared_pfx_ref = shared_pfx_ref.clone();

            let cfg_compat_tool = get_compat_tool_from_config();
            if cfg_compat_tool.is_none() {
                show_message_dialog("No compat tool selected.");
                return;
            }

            tokio::spawn(async move {
                if which("winetricks").is_err() {
                    show_message_dialog("Could not find winetricks in the system path.");
                } else {
                    let borrowed_pfx_ref = shared_pfx_ref.lock().await;
                    let run_winetricks = borrowed_pfx_ref.run_wiretricks(&cfg_compat_tool.unwrap()).await;
                    if run_winetricks.is_err() {
                        show_message_dialog("Winetricks failed to run");
                    }
                }
                let _ = weak_window.upgrade_in_event_loop(|window| {
                    window.set_running_winetricks(false);
                });
            });
        }
    });
    
    let _ = window.show();
}
