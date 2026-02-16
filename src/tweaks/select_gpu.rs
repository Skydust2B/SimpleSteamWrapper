use std::process::Command;
use log::{info};
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

    env_vars.push((
        "VK_LOADER_DEVICE_SELECT".to_string(),
        gpu.as_formatted_id()
    ));

    env_vars.push((
        "MESA_VK_DEVICE_SELECT".to_string(),
        gpu.as_formatted_id()
    ));

    env_vars
}

pub fn get_nvidia_gpu_env_vars(gpu: &GPU) -> Vec<(String,String)> {
    let mut env_vars = Vec::<(String, String)>::new();

    info!("Retrieving NVIDIA GPU UUID");
    if let Some(uuid) = get_nvidia_gpu_uuid(gpu).ok() {
        info!("Found NVIDIA GPU UUID: {}", uuid);
        env_vars.push((
            "CUDA_VISIBLE_DEVICES".to_string(),
            uuid
        ));
    }

    env_vars
}

pub fn get_gpu_select_env_vars(gpu: &GPU) -> Vec<(String, String)> {
    let mut env_vars = Vec::<(String, String)>::new();
    if gpu.vendor_id == 0x10DE {
        env_vars = [env_vars, get_nvidia_gpu_env_vars(gpu)].concat();
    }
    env_vars = [env_vars, get_vulkan_gpu_env_vars(&gpu)].concat();

    env_vars.push(("DRI_PRIME".to_string(), gpu.as_formatted_id()));

    env_vars
}
