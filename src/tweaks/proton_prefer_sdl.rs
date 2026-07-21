use std::process::Command;
use tweaks_macro::tweak;
use crate::tweak_collector::PreparedCommand;

#[tweak(name = "proton_prefer_sdl")]
pub fn run(process: &mut Command, _: &mut PreparedCommand) {
    process.env("PROTON_PREFER_SDL", "1");
}
