use std::process::Command;
use log::{info};
use tweaks_macro::tweak;
use crate::gpu_tools::gpu::{get_gpu_from_config, GPU};
use crate::gpu_tools::nvidia_gpu::get_nvidia_gpu_uuid;

#[tweak(name = "nvidia_vram_workarounds", priority=0)]
pub fn run(process: &mut Command, _: &mut Vec<String>) {
    let gpu_to_set = &get_gpu_from_config();
    info!("Using selected GPU: {}", gpu_to_set.full_name);

    let mut env_vars = Vec::<(String, String)>::new();

    // Excessive VRAM on DX12 with resizable BAR configurations.
    env_vars.push((
        "VKD3D_CONFIG".to_string(),
        "no_upload_hvv".to_string()
    ));

    // DXVK soft VRAM limit
    env_vars.push((
        "DXVK_CONFIG".to_string(),
        "dxgi.maxDeviceMemory = 6144;".to_string()
    ));

    process.envs(env_vars);
}
