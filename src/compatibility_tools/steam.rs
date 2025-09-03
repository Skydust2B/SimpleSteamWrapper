use std::fs;
use std::path::PathBuf;
use std::string::ToString;
use vdf_reader::entry::{Entry};
use crate::compatibility_tools::compat_tools_wrapper::{CompatTool};
use crate::compatibility_tools::installed_steam_apps::{get_installed_steam_apps, InstalledSteamGame};
use crate::vdf_tools::vdf_simple_parser::read_vdf;

const STEAM_VALID_COMPAT_APPIDS: [&str; 14] = [
    "2230260",
    "2180100",
    "1493710",
    "3658110",
    "2805730",
    "2348590",
    "1887720",
    "1580130",
    "1420170",
    "1245040",
    "1113280",
    "1054830",
    "961940",
    "858280",
];

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

pub fn get_steam_sniper_runtime() -> Option<InstalledSteamGame> {
    let steam_apps = get_installed_steam_apps();
    if let Some(app) = steam_apps.get("1628350") {
        return Some(app.clone());
    }
    None
}

pub fn get_steam_compat_tools_path() -> PathBuf {
    PathBuf::from(get_steam_path().unwrap()).join("compatibilitytools.d")
}

pub fn parse_steam_compat_tool_from_app(app: InstalledSteamGame) -> CompatTool {
    let tool_manifest = read_vdf(app.path.join("toolmanifest.vdf"));

    let command_path = app.path.join(
        tool_manifest["manifest"]
            .get("commandline")
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| "")
            .to_string()
            .strip_prefix("/").unwrap_or_else(|| "")
    );

    CompatTool {
        name: app.name.to_string(),
        dir_path: app.path.to_str().unwrap().to_string(),
        path: command_path.to_str().unwrap().to_string().replace(" %verb%", "")
    }
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
        path: command_path.to_str().unwrap().to_string().replace(" %verb%", "")
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

    let steam_apps = get_installed_steam_apps();
    STEAM_VALID_COMPAT_APPIDS.iter().for_each(|app_id| {
        if let Some(app) = steam_apps.get(&app_id.to_string()) {
            results.push(parse_steam_compat_tool_from_app(app.clone()));
        }
    });

    results
}
