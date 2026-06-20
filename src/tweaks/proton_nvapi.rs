use std::process::Command;
use tweaks_macro::tweak;
use crate::tweak_collector::PreparedCommand;

#[tweak(name = "proton_nvapi")]
pub fn run(process: &mut Command, _: &mut PreparedCommand) {
    process.env("PROTON_ENABLE_NVAPI", "1");
    process.env("PROTON_FORCE_NVAPI", "1");
}
