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
    pub force_grab_cursor: bool
}

#[tweak(name = "gamescope")]
pub fn run(_: &mut Command, prepared_command: &mut Vec<String>) {
    let mut index = 0;
    let mut gamescope_cmd = vec!["gamescope"];

    let settings = LOADED_CONFIG.get_config().defaults.gamescope_settings;
    if settings.fullscreen {
        gamescope_cmd.push("-f");
    }
    if settings.force_grab_cursor {
        gamescope_cmd.push("--force-grab-cursor")
    }
    if settings.forced_width > 0 {
        gamescope_cmd.push("-W");
        gamescope_cmd.push(settings.forced_width.to_string().as_str());
    }
    if settings.forced_height > 0 {
        gamescope_cmd.push("-H");
        gamescope_cmd.push(settings.forced_height.to_string().as_str());
    }
    let _ = &["gamescope", "-f", "-W", "1920", "-H", "1080", "-r", "165", "--"]
        .iter()
        .for_each(|v| {
            prepared_command
                .insert(index, v.to_string());

            index = index + 1;
        });
}
