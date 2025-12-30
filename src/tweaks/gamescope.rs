use std::process::Command;
use serde::{Deserialize, Serialize};
use tweaks_macro::tweak;
use crate::config::config_loader::LOADED_CONFIG;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GamescopeSettings {
    pub forced_width: i32,
    pub forced_height: i32,
    pub fullscreen: bool,
    pub framerate: i32,
    pub force_grab_cursor: bool,
    pub hdr: bool
}

#[tweak(name = "gamescope", priority = 100)]
pub fn run(_: &mut Command, prepared_command: &mut Vec<String>) {
    let mut gamescope_cmd: Vec<String> = vec!["gamescope".to_string()];

    let settings = LOADED_CONFIG.get_app_options().gamescope_settings;
    if settings.fullscreen {
        gamescope_cmd.push("-f".to_string());
    }
    if settings.force_grab_cursor {
        gamescope_cmd.push("--force-grab-cursor".to_string())
    }
    if settings.forced_width > 0 {
        gamescope_cmd.push("-W".to_string());
        gamescope_cmd.push(settings.forced_width.to_string());
    }
    if settings.forced_height > 0 {
        gamescope_cmd.push("-H".to_string());
        gamescope_cmd.push(settings.forced_height.to_string());
    }
    if settings.framerate > 0 {
        gamescope_cmd.push("-r".to_string());
        gamescope_cmd.push(settings.framerate.to_string());
    }
    if settings.hdr {
        gamescope_cmd.push("--hdr-enabled".to_string());
    }
    gamescope_cmd.push("--".to_string());

    let mut index = 0;
    let _ = gamescope_cmd
        .iter()
        .for_each(|v| {
            prepared_command
                .insert(index, v.to_string());
            index = index + 1;
        });
}
