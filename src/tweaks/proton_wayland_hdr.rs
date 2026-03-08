use std::process::Command;
use tweaks_macro::tweak;
use crate::gpu_tools::gpu::GPU;

#[tweak(name = "proton_wayland_hdr")]
pub fn run(process: &mut Command, _: &mut Vec<String>) {
    process.env("PROTON_ENABLE_HDR", "1");

    if GPU::from_config().is_nvidia() {
        process.env("ENABLE_HDR_WSI", "1");
    }
}
