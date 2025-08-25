use std::process::Command;
use tweaks_macro::tweak;

#[tweak(name = "proton_nvapi")]
pub fn run(process: &mut Command, _: &mut Vec<String>) {
    process.env("PROTON_ENABLE_NVAPI", "1");
}
