use std::env;
use std::path::PathBuf;
use rfd::FileDialog;
use slint::{ComponentHandle, SharedString};
use crate::compatibility_tools::compat_tools_wrapper::{get_compat_tool_from_config, reset_prefix, run_in_prefix, run_wiretricks_in_prefix};
use crate::config::config_loader::get_steam_app_id;
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
                run_in_prefix(path, in_terminal);
            }
        });

    window.on_recreate_prefix(|| {
        reset_prefix();
    });

    window.on_run_winetricks(|| {
        run_wiretricks_in_prefix();
    });
    
    let _ = window.run();
}
