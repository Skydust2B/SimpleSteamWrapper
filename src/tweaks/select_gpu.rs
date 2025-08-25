use std::process::Command;
use log::info;
use tweaks_macro::tweak;
use crate::gpu::{get_formatted_gpu_id, get_gpu_from_config, GPU};

#[tweak(name = "select_gpu", priority=0)]
pub fn run(process: &mut Command, _: &mut Vec<String>) {
    let gpu_to_set = &get_gpu_from_config();
    info!("Using selected GPU: {}", gpu_to_set.full_name);

    process.envs(get_gpu_select_env_vars(gpu_to_set));
}

// Gets a hashmap with various env vars made to force the selection of a specific GPU
pub fn get_gpu_select_env_vars(gpu: &GPU) -> Vec<(String, String)> {
    let mut env_vars = Vec::<(String, String)>::new();

    env_vars.push((
        "VK_LOADER_DEVICE_SELECT".to_string(),
        get_formatted_gpu_id(gpu)
    ));
    env_vars.push((
        "DXVK_FILTER_DEVICE_NAME".to_string(),
        gpu.name.clone()
    ));
    env_vars.push((
        "VKD3D_FILTER_DEVICE_NAME".to_string(),
        gpu.name.clone()
    ));

    env_vars
}
