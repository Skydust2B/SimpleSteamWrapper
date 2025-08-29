use log::warn;
use pci_ids::{FromId, Vendor};
use pci_info::PciInfo;
use pci_info::pci_enums::PciDeviceClass;
use regex::Regex;
use crate::config::config_loader::LOADED_CONFIG;

#[derive(Debug, Clone)]
pub struct GPU {
    pub name: String,
    pub full_name: String,
    pub vendor_id: u16,
    pub device_id: u16
}

impl GPU {
    pub fn as_formatted_id(&self) -> String {
        format!("0x{:04x}:0x{:04x}", self.vendor_id, self.device_id)
    }
}

pub fn get_gpu_from_config() -> GPU {
    let cfg = LOADED_CONFIG.get_app_options();
    let all_gpu = list_all_gpus();

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

pub fn list_all_gpus() -> Vec<GPU> {
    // Enumerate all PCI devices
    let devices = PciInfo::enumerate_pci().unwrap();

    let mut valid_gpus = Vec::<GPU>::new();

    for dev in devices {
        if let Ok(device) = dev {
            let vendor_id = device.vendor_id();
            let device_id = device.device_id();
            let class_id = device.device_class().unwrap();

            if class_id == PciDeviceClass::DisplayController {
                let vendor = Vendor::from_id(vendor_id).unwrap();
                let device = vendor.devices().find(|v| v.id() == device_id).unwrap();
                let device_name = device.name();

                let re = Regex::new(r"\[(.*?)]").unwrap();
                let gpu_name = re.captures(device_name)
                    .and_then(|v| v.get(1))
                    .map(|m| m.as_str().to_string())
                    .unwrap_or_else(|| device_name.to_string());

                valid_gpus.push(
                    GPU {
                        full_name: format!("{} {}", vendor.name(), gpu_name.to_string()),
                        name: gpu_name,
                        vendor_id: vendor.id(),
                        device_id: device.id()
                    }
                )
            }
        }
    }

    valid_gpus
}
