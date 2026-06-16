use std::process::Command;
use std::str::FromStr;
use anyhow::anyhow;
use log::{info};
use crate::gpu_tools::gpu::GPU;

#[derive(Clone)]
pub struct NvidiaGPUData {
    device_id: u32,
    pub(crate) uuid: String,
    pub total_memory_mb: i32
}

pub fn get_nvidia_gpu_data(gpu: &GPU) -> anyhow::Result<NvidiaGPUData> {
    let result = Command::new("nvidia-smi")
        .arg("--query-gpu=pci.device_id,uuid,memory.total")
        .arg("--format=csv,noheader,nounits")
        .output()?;

    if result.status.success() {
        info!("Retrieving data from nvidia-smi");
        let stdout = String::from_utf8_lossy(&result.stdout);
        let parsed_gpus = stdout.lines()
            .map(|s| {
                let line = s.split(", ").collect::<Vec<&str>>();
                NvidiaGPUData {
                    device_id: u32::from_str_radix(line[0].trim_start_matches("0x"), 16).unwrap_or_default(),
                    uuid: line[1].to_string(),
                    total_memory_mb: i32::from_str(line[2]).unwrap_or(0)
                }
            }).collect::<Vec<NvidiaGPUData>>();

        if let Some(found_gpu) = parsed_gpus.iter().find(|nvidia_gpu|
            (nvidia_gpu.device_id >> 16) as u16 == gpu.device_id) {
            return Ok(found_gpu.clone());
        }
    }
    Err(anyhow!("Unable to find the NVIDIA GPU UUID"))
}
