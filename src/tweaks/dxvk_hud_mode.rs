use std::process::Command;
use serde::{Deserialize, Serialize};
use tweaks_macro::tweak;
use crate::config::global_config::{GlobalConfig};
use crate::tweak_collector::PreparedCommand;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DXVKHUDSettings {
    pub mode: String
}

#[tweak(name = "dxvk_hud_mode")]
pub fn run(process: &mut Command, _: &mut PreparedCommand) {
    let dxvk_hud_settings = GlobalConfig::get_app_options().dxvk_hud_settings;
    process.env("DXVK_HUD", dxvk_hud_settings.mode);
}
