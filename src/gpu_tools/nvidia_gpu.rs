use std::process::Command;
use anyhow::anyhow;
use log::{info};
use crate::gpu_tools::gpu::GPU;

pub fn get_nvidia_gpu_uuid(gpu: &GPU) -> anyhow::Result<String> {
    let result = Command::new("nvidia-smi")
        .arg("--query-gpu=pci.device_id,uuid")
        .arg("--format=csv,noheader")
        .output()?;

    if result.status.success() {
        info!("Retrieving data from nvidia-smi");
        let stdout = String::from_utf8_lossy(&result.stdout);
        let parsed_gpus = stdout.lines()
            .map(|s| {
                let line = s.split(", ").collect::<Vec<&str>>();
                (u32::from_str_radix(line[0].trim_start_matches("0x"), 16)
                     .unwrap_or_default(),
                 line[1].to_string())
            }).collect::<Vec<(u32, String)>>();

        if let Some((_, uuid)) = parsed_gpus.iter().find(|(d_id, _uuid)|
            (d_id >> 16) as u16 == gpu.device_id) {
            return Ok(uuid.to_string());
        }
    }
    Err(anyhow!("Unable to find the NVIDIA GPU UUID"))
}
