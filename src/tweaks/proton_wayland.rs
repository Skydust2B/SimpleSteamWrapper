use std::process::Command;
use tweaks_macro::tweak;
use crate::tweak_collector::PreparedCommand;

#[tweak(name = "proton_wayland")]
pub fn run(process: &mut Command, _: &mut PreparedCommand) {
    process.env("PROTON_ENABLE_WAYLAND", "1");
}
