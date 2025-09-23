use std::process::Command;
use log::{info, warn};
use tweaks_macro::tweak;
use crate::gpu_tools::gpu::{GPU};
use crate::gpu_tools::nvidia_gpu::get_nvidia_gpu_uuid;

#[tweak(name = "gpu_tools", priority=0)]
pub fn run(process: &mut Command, _: &mut Vec<String>) {
    let gpu_to_set = GPU::from_config();
    info!("Using selected GPU: {}", gpu_to_set.full_name);

    process.envs(get_gpu_select_env_vars(&gpu_to_set));
}

pub fn get_vulkan_gpu_env_vars(gpu: &GPU) -> Vec<(String,String)> {
    let mut env_vars = Vec::<(String, String)>::new();

    // Any vulkan app
    env_vars.push((
        "VK_LOADER_DEVICE_SELECT".to_string(),
        gpu.as_formatted_id()
    ));

    // Mesa device selection
    env_vars.push((
        "MESA_VK_DEVICE_SELECT".to_string(),
        gpu.as_formatted_id()
    ));

    // DXVK device selection
    env_vars.push((
        "DXVK_FILTER_DEVICE_NAME".to_string(),
        gpu.name.clone()
    ));

    // DX12 device selection
    env_vars.push((
        "VKD3D_FILTER_DEVICE_NAME".to_string(),
        gpu.name.clone()
    ));

    env_vars
}

pub fn get_nvidia_gpu_env_vars(gpu: &GPU) -> Vec<(String,String)> {
    let mut env_vars = Vec::<(String, String)>::new();

    info!("Retrieving NVIDIA GPU UUID");
    if let Some(uuid) = get_nvidia_gpu_uuid(gpu).ok() {
        info!("Found NVIDIA GPU UUID: {}", uuid);
        env_vars.push((
            "CUDA_VISIBLE_DEVICES".to_string(), // For cuda application, official var
            uuid.clone()
        ));

        env_vars.push((
            "__NV_PRIME_RENDER_OFFLOAD".to_string(),
            "1".to_string()
        ));
        env_vars.push((
            "__GLX_VENDOR_LIBRARY_NAME".to_string(),
            "nvidia".to_string()
        ));
        env_vars.push((
            "__NV_PRIME_RENDER_OFFLOAD_PROVIDER".to_string(), // Isn't documented to work for wayland and isn't supposed to accept UUIDs, but some rumors had them supposedly work anyway
            uuid
        ));
    } else {
        warn!("GPU UUID not found, is the driver working ?");
    }

    env_vars
}

pub fn get_gpu_select_env_vars(gpu: &GPU) -> Vec<(String, String)> {
    let mut env_vars = Vec::<(String, String)>::new();
    if gpu.is_nvidia() {
        env_vars = [env_vars, get_nvidia_gpu_env_vars(gpu)].concat();
    }
    env_vars = [env_vars, get_vulkan_gpu_env_vars(&gpu)].concat();

    // Main GPU selector
    env_vars.push(("DRI_PRIME".to_string(), gpu.as_formatted_id()));

    env_vars
}

