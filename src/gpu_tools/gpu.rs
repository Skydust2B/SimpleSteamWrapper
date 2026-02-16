use log::{warn};
use crate::config::global_config::GlobalConfig;
use crate::gpu_tools::gpu_list::GPUList;

#[derive(Debug, Clone)]
pub struct GPU {
    pub full_name: String,
    pub vendor_id: u16,
    pub device_id: u16
}

impl GPU {
    pub fn as_formatted_id(&self) -> String {
        format!("0x{:04x}:0x{:04x}", self.vendor_id, self.device_id)
    }

    pub fn is_nvidia(&self) -> bool {
        self.vendor_id == 0x10DE
    }

    pub fn from_config() -> Self {
        let cfg = GlobalConfig::get_app_options();
        let all_gpu = GPUList::get();

        if all_gpu.len() == 0 {
            panic!("Unable to find a GPU")
        }

        let retrieved_gpu = all_gpu.iter().find(|gpu| gpu.as_formatted_id() == cfg.selected_gpu);
        if retrieved_gpu.is_none() {
            let found_gpu = all_gpu.first().unwrap().clone();
            warn!("Unable to find selected GPU, using {}", found_gpu.full_name);
            return found_gpu;
        }
        retrieved_gpu.unwrap().clone()
    }
}
