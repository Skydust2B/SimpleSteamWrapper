use std::process::Command;
use tweaks_macro::tweak;

#[tweak(name = "dxvk_hud_mode")]
pub fn run(process: &mut Command, _: &mut Vec<String>) {
    process.env("DXVK_HUD", "compiler");
}
