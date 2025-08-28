use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::tweaks::dxvk_hud_mode::DXVKHUDSettings;
use crate::tweaks::gamescope::GamescopeSettings;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub general: General,
    pub defaults: Options,
    pub apps: HashMap<String, Options>
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct General {
    pub theme: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Options {
    pub selected_gpu: String,
    pub compat_tool: String,
    pub gamescope_settings: GamescopeSettings,
    pub dxvk_hud_settings: DXVKHUDSettings,
    pub enabled_tweaks: HashMap<String, bool>,
}

impl Config {
    pub(crate) fn new() -> Self {
        Self {
            general: General {
                theme: "dark".to_string(),
            },
            defaults: Options {
                selected_gpu: "".to_string(),
                enabled_tweaks: HashMap::new(),
                compat_tool: "".to_string(),
                gamescope_settings: GamescopeSettings {
                    forced_width: 1920,
                    forced_height: 1080,
                    force_grab_cursor: true,
                    framerate: 165,
                    fullscreen: true
                },
                dxvk_hud_settings: DXVKHUDSettings {
                    mode: "compiler".to_string()
                }
            },
            apps: HashMap::new()
        }
    }
}
