use std::process::Command;
use tweaks_macro::tweak;

#[tweak(name = "proton_nvapi_vkreflex")]
pub fn run(process: &mut Command, _: &mut Vec<String>) {
    process.env("DXVK_NVAPI_VKREFLEX", "1");
}
