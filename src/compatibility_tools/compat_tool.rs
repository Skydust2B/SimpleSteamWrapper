use std::path::PathBuf;
use log::{warn};
use crate::compatibility_tools::compat_tools_list::{CompatToolsList};
use crate::config::global_config::{GlobalConfig};

#[derive(Debug, Clone)]
pub struct CompatTool {
    pub name: String,
    pub dir_path: String,
    pub path: String
}

impl CompatTool {
    pub fn find_wine_bin(&self) -> Option<PathBuf> {
        let dir_path = PathBuf::from(self.clone().dir_path);
        let candidates = [
            dir_path.join("files/bin/wine"),
            dir_path.join("bin/wine"),
            dir_path.join("usr/bin/wine")
        ];

        candidates.into_iter().find(|p| p.exists())
    }
}

pub fn get_compat_tool_from_config() -> Option<CompatTool> {
    let cfg = GlobalConfig::get_app_options();
    let all_ct = CompatToolsList::get();

    let retrieved_ct = all_ct.iter().find(|ct| cfg.compat_tool == ct.name);
    if cfg.compat_tool == "" {
        return None;
    }
    if retrieved_ct.is_none() {
        warn!("Unable to find selected compatibility tool {}", cfg.compat_tool);
        return None;
    }
    Some(retrieved_ct.unwrap().clone())
}
