use std::{env};
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

pub fn get_wine_variables() -> Vec<(String, String)> {
    let mut env_vars = Vec::<(String, String)>::new();
    let data_path = env::var("STEAM_COMPAT_DATA_PATH").expect("STEAM_COMPAT_DATA_PATH must be set");

    env_vars.push(("WINE_PREFIX".to_string(), PathBuf::from(data_path).join("pfx").to_str().unwrap().to_string()));

    let game_data_path = env::var("STEAM_COMPAT_INSTALL_PATH").expect("STEAM_COMPAT_INSTALL_PATH must be set");
    env_vars.push(("PWD".to_string(), game_data_path));

    env_vars.push(("WINEDEBUG".to_string(),"-all".to_string()));

    env_vars
}
