use std::fs;
use std::path::PathBuf;
use log::warn;
use vdf_reader::entry::{Entry, Table};
use vdf_reader::from_str;
use crate::config::config_loader::LOADED_CONFIG;
use crate::gpu::{get_formatted_gpu_id, list_all_gpus, GPU};

pub fn get_steam_path() -> Option<PathBuf> {
    let home = std::env::var_os("HOME")?;
    let base = PathBuf::from(home);
    // Common Linux locations (the first usually exists)
    let candidates = [
        base.join(".local/share/Steam"),
        base.join(".steam/steam"),
    ];
    candidates.into_iter().find(|p| p.exists())
}

pub fn get_compat_tool_from_config() -> CompatTool {
    let cfg = LOADED_CONFIG.get_config();
    let all_ct = list_steam_compat_tools();

    if all_ct.len() == 0 {
        panic!("Unable to find a compatibility tool, use ProtonUpQt to download some.")
    }

    let retrieved_ct = all_ct.iter().find(|ct| cfg.defaults.compat_tool == ct.name);
    if retrieved_ct.is_none() {
        let found_ct = all_ct.first().unwrap().clone();
        warn!("Unable to find selected compatibility tool, using {}", found_ct.name);
        return found_ct;
    }
    retrieved_ct.unwrap().clone()
}

pub fn get_steam_config() -> Table {
    let steam_path = get_steam_path().unwrap();
    let steam_config_path = steam_path.join("config/config.vdf");
    let text = fs::read_to_string(steam_config_path).unwrap();

    from_str(&text).unwrap()
}

pub fn get_steam_default_compat_tool() -> String {
    let cfg = get_steam_config();

    cfg.get("InstallConfigStore")
        .and_then(|v| v.get("Software"))
        .and_then(|v| v.get("Valve"))
        .and_then(|v| v.get("Steam"))
        .and_then(|v| v.get("CompatToolMapping"))
        .and_then(|v| v.get("0")) // “0” is the default mapping
        .and_then(|v| v.get("name"))
        .and_then(|v| v.as_str())
        .unwrap_or_else(|| "").to_string()
}

#[derive(Debug, Clone)]
pub struct CompatTool {
    pub name: String,
    pub dir_path: String,
    pub path: String
}

pub fn parse_steam_compat_tool(path: PathBuf) -> CompatTool {
    let comp_tool_infos_path = path.join("compatibilitytool.vdf");
    let raw_comp_tool_infos = fs::read_to_string(comp_tool_infos_path).unwrap();
    let comp_tool_infos: Table = from_str(&raw_comp_tool_infos).unwrap();

    let data: &Entry = &comp_tool_infos.get("compatibilitytools")
        .and_then(|v| v.get("compat_tools"))
        .and_then(|v| v.as_table())
        .and_then(|v| v.values().next())
        .unwrap();

    let compat_tool_dir_path = path.join(
        data.get("install_path")
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| "")
    ).canonicalize().unwrap();

    let compat_tool_display_name = data.get("display_name").unwrap().as_str().unwrap_or_else(|| "");

    let tool_manifest_path = &compat_tool_dir_path.join("toolmanifest.vdf");
    let raw_tool_manifest = fs::read_to_string(tool_manifest_path).unwrap();
    let tool_manifest: Table = from_str(&raw_tool_manifest).unwrap();

    let command_path = compat_tool_dir_path.join(
        tool_manifest.get("manifest")
            .and_then(|v| v.get("commandline"))
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| "")
            .to_string().strip_prefix("/").unwrap_or_else(|| "")
    );

    CompatTool {
        name: compat_tool_display_name.to_string(),
        dir_path: compat_tool_dir_path.to_str().unwrap().to_string(),
        path: command_path.to_str().unwrap().to_string()
    }
}

pub fn list_steam_compat_tools() -> Vec<CompatTool> {
    let steam_path = get_steam_path().unwrap();
    let steam_compat_tools_path = PathBuf::from(steam_path).join("compatibilitytools.d");

    let mut results = vec![];
    if let Ok(entries) = fs::read_dir(steam_compat_tools_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                results.push(parse_steam_compat_tool(path));
            }
        }
    }
    results
}
