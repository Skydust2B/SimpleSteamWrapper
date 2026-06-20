use std::process::Command;
use tweaks_macro::tweak;
use crate::tweak_collector::PreparedCommand;

#[tweak(name = "proton_nvapi_vkreflex")]
pub fn run(process: &mut Command, _: &mut PreparedCommand) {
    process.env("DXVK_NVAPI_VKREFLEX", "1");
}
