use std::collections::HashMap;
use device_query::Keycode;
use serde::{Deserialize, Serialize};
use crate::tweaks::dxvk_hud_mode::DXVKHUDSettings;
use crate::tweaks::gamescope::GamescopeSettings;
use crate::runner::runtime::Runtime;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub general: General,
    pub defaults: Options,
    pub apps: HashMap<String, Options>
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct General {
    pub gui_trigger_key: String,
    pub show_on_game_crash: bool
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Options {
    pub selected_gpu: String,
    pub compat_tool: String,
    pub runtime: Runtime,
    pub gamescope_settings: GamescopeSettings,
    pub dxvk_hud_settings: DXVKHUDSettings,
    pub enabled_tweaks: HashMap<String, bool>,
    pub custom_env_vars: HashMap<String, String>
}

impl Config {
    pub(crate) fn new() -> Self {
        Self {
            general: General {
                gui_trigger_key: Keycode::LShift.to_string(),
                show_on_game_crash: false,
            },
            defaults: Options {
                selected_gpu: "".to_string(),
                enabled_tweaks: HashMap::new(),
                runtime: Runtime::SteamSniper, // Should use sniper by default
                compat_tool: "".to_string(),
                gamescope_settings: GamescopeSettings {
                    forced_width: 1920,
                    forced_height: 1080,
                    force_grab_cursor: true,
                    framerate: 165,
                    fullscreen: true,
                    hdr: false
                },
                dxvk_hud_settings: DXVKHUDSettings {
                    mode: "compiler".to_string()
                },
                custom_env_vars: HashMap::new()
            },
            apps: HashMap::new()
        }
    }
}
