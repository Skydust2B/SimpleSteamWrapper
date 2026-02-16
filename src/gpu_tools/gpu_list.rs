use std::sync::{Arc, Mutex};
use log::debug;
use once_cell::sync::Lazy;
use pci_ids::{FromId, Vendor};
use pci_info::pci_enums::PciDeviceClass;
use pci_info::PciInfo;
use regex::Regex;
use crate::gpu_tools::gpu::GPU;

fn retrieve_all_gpus() -> Vec<GPU> {
    let devices = PciInfo::enumerate_pci().unwrap();

    let mut valid_gpus = Vec::<GPU>::new();

    for dev in devices {
        if let Ok(device) = dev {
            let class_id = device.device_class().unwrap();
            if class_id != PciDeviceClass::DisplayController { continue; }

            let vendor_id = device.vendor_id();
            let device_id = device.device_id();

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
                    vendor_id: vendor.id(),
                    device_id: device.id()
                }
            )
        }
    }

    valid_gpus
}

#[derive(Clone)]
pub(crate) struct GPUList {
    state: Arc<Mutex<Vec<GPU>>>,
}

impl GPUList {
    fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn get_internal(&self) -> Vec<GPU> {
        let state = self.state.lock().unwrap();
        state.clone()
    }

    pub(crate) fn get() -> Vec<GPU> {
        let mut list = GPU_LIST.get_internal();
        if list.is_empty() {
            Self::refresh();
            list = GPU_LIST.get_internal();
        }
        list
    }

    pub(crate) fn refresh() {
        let mut state = GPU_LIST.state.lock().unwrap();
        let gpus = retrieve_all_gpus();
        debug!("Found {} gpus", gpus.len());
        *state = gpus;
    }
}

static GPU_LIST: Lazy<GPUList> = Lazy::new(GPUList::new);
