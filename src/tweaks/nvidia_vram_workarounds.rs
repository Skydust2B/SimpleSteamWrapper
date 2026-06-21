use std::process::Command;
use tweaks_macro::tweak;
use crate::gpu_tools::gpu::{GPU};
use crate::gpu_tools::nvidia_gpu::get_nvidia_gpu_data;
use crate::tweak_collector::PreparedCommand;
use crate::utils::command_utils::UpdateEnvVar;

#[tweak(name = "nvidia_vram_workarounds")]
pub fn run(process: &mut Command, _: &mut PreparedCommand) {
    let gpu = GPU::from_config();

    // Excessive VRAM on DX12 with resizable BAR configurations.
    process.add_parameter_to_var(",", "VKD3D_CONFIG", "no_upload_hvv");

    if let Some(nvidia_gpu) = get_nvidia_gpu_data(&gpu).ok() {
        let soft_vram_limit = ((nvidia_gpu.total_memory_mb as f32) * 0.9).floor() as i32;

        // DXVK soft VRAM limit
        process.add_parameter_to_var(";", "DXVK_CONFIG", &format!("dxgi.maxDeviceMemory = {}", soft_vram_limit));
    }
}
