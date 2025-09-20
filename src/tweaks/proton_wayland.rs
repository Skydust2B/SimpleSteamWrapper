use std::process::Command;
use tweaks_macro::tweak;

#[tweak(name = "proton_wayland")]
pub fn run(process: &mut Command, _: &mut Vec<String>) {
    process.env("PROTON_ENABLE_WAYLAND", "1");
}
