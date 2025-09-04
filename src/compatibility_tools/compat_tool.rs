use std::path::PathBuf;
use log::{warn};
use crate::compatibility_tools::steam::list_steam_compat_tools;
use crate::config::config_loader::LOADED_CONFIG;

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

pub fn get_compat_tool_from_config() -> CompatTool {
    let cfg = LOADED_CONFIG.get_app_options();
    let all_ct = list_steam_compat_tools();

    if all_ct.len() == 0 {
        panic!("Unable to find a compatibility tool, use ProtonUpQt to download some.")
    }

    let retrieved_ct = all_ct.iter().find(|ct| cfg.compat_tool == ct.name);
    if retrieved_ct.is_none() {
        let found_ct = all_ct.first().unwrap().clone();
        warn!("Unable to find selected compatibility tool, using {}", found_ct.name);
        return found_ct;
    }
    retrieved_ct.unwrap().clone()
}
