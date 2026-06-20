use std::process::Command;
use tweaks_macro::tweak;
use crate::gpu_tools::gpu::{GPU};
use crate::gpu_tools::nvidia_gpu::get_nvidia_gpu_data;
use crate::tweak_collector::PreparedCommand;

#[tweak(name = "nvidia_vram_workarounds")]
pub fn run(process: &mut Command, _: &mut PreparedCommand) {
    let gpu = GPU::from_config();

    let mut env_vars = Vec::<(String, String)>::new();

    // Excessive VRAM on DX12 with resizable BAR configurations.
    env_vars.push((
        "VKD3D_CONFIG".to_string(),
        "no_upload_hvv".to_string()
    ));

    if let Some(nvidia_gpu) = get_nvidia_gpu_data(&gpu).ok() {
        let soft_vram_limit = ((nvidia_gpu.total_memory_mb as f32) * 0.9).floor() as i32;

        // DXVK soft VRAM limit
        env_vars.push((
            "DXVK_CONFIG".to_string(),
            format!("dxgi.maxDeviceMemory = {};", soft_vram_limit)
        ));
    }

    process.envs(env_vars);
}
