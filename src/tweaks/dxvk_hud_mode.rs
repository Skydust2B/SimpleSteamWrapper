use std::process::Command;
use serde::{Deserialize, Serialize};
use tweaks_macro::tweak;
use crate::config::config_loader::LOADED_CONFIG;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DXVKHUDSettings {
    pub mode: String
}

#[tweak(name = "dxvk_hud_mode")]
pub fn run(process: &mut Command, _: &mut Vec<String>) {
    let dxvk_hud_settings = LOADED_CONFIG.get_app_options().dxvk_hud_settings;
    process.env("DXVK_HUD", dxvk_hud_settings.mode);
}
