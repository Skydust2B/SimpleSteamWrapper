use std::fs;
use std::path::PathBuf;
use vdf_reader::entry::{Entry, Table};
use crate::compatibility_tools::compat_tools_wrapper::{CompatTool};
use crate::vdf_tools::vdf_simple_parser::read_vdf;

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

pub fn get_steam_config() -> Table {
    let steam_path = get_steam_path().unwrap();
    let steam_config_path = steam_path.join("config/config.vdf");
    read_vdf(steam_config_path)
}

pub fn get_steam_default_compat_tool() -> String {
    let cfg = get_steam_config();

    cfg["InstallConfigStore"].lookup("Software.Valve.Steam.CompatToolMapping.0.name")
        .and_then(|v| v.as_str())
        .unwrap_or_else(|| "").to_string()
}

pub fn get_steam_compat_tools_path() -> PathBuf {
    PathBuf::from(get_steam_path().unwrap()).join("compatibilitytools.d")
}

pub fn parse_steam_compat_tool(path: PathBuf) -> CompatTool {
    let compat_tool_vdf = read_vdf(path.join("compatibilitytool.vdf"));
    let compat_tool_data: &Entry = compat_tool_vdf["compatibilitytools"]
        .get("compat_tools")
        .and_then(|v| v.as_table())
        .and_then(|v| v.values().next())
        .unwrap();

    let compat_tool_dir_path = path.join(
        compat_tool_data.get("install_path")
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| "")
    ).canonicalize().unwrap();

    let compat_tool_display_name = compat_tool_data.get("display_name").unwrap().as_str().unwrap_or_else(|| "Borken");

    let tool_manifest = read_vdf(path.join("toolmanifest.vdf"));

    let command_path = compat_tool_dir_path.join(
        tool_manifest["manifest"]
            .get("commandline")
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| "")
            .to_string()
            .strip_prefix("/").unwrap_or_else(|| "")
    );

    CompatTool {
        name: compat_tool_display_name.to_string(),
        dir_path: compat_tool_dir_path.to_str().unwrap().to_string(),
        path: command_path.to_str().unwrap().to_string()
    }
}

pub fn list_steam_compat_tools() -> Vec<CompatTool> {
    let steam_compat_tools_path = get_steam_compat_tools_path();

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
