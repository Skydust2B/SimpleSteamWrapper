use std::{env, fs};
use std::path::PathBuf;
use std::string::ToString;
use anyhow::{anyhow, Context};
use vdf_reader::entry::{Entry, Table};
use crate::compatibility_tools::compat_tool::{CompatTool};
use crate::steam::installed_steam_apps::{get_installed_steam_apps, InstalledSteamApp};
use crate::runner::runtime::Runtime;

const STEAM_VALID_COMPAT_APPIDS: [&str; 15] = [
    "2230260", // Proton Next
    "2180100", // Proton Hotfix
    "1493710", // Proton Experimental
    "4628710", // Proton 11.0
    "3658110", // Proton 10.0
    "2805730", // Proton 9.0
    "2348590", // Proton 8.0
    "1887720", // Proton 7.0
    "1580130", // Proton 6.3
    "1420170", // Proton 5.13
    "1245040", // Proton 5.0
    "1113280", // Proton 4.11
    "1054830", // Proton 4.2
    "961940", // Proton 3.16
    "858280", // Proton 3.7
];

pub fn read_vdf(path: &PathBuf) -> anyhow::Result<Table> {
    let text = fs::read_to_string(&path)
        .with_context(|| format!("Unable to find {}", path.display()))?;
    Ok(Table::load_from_str(&text)?)
}

pub fn get_steam_path() -> Option<PathBuf> {
    let home = env::var_os("HOME")?;
    let base = PathBuf::from(home);
    // Common Linux locations (the first usually exists)
    let candidates = [
        base.join(".local/share/Steam"),
        base.join(".steam/steam"),
    ];
    let path = candidates.into_iter().find(|p| p.exists());
    if path.is_none() {
        panic!("Can't find steam folder");
    }
    path
}

pub fn get_steam_runtime_app(runtime: Runtime) -> Option<InstalledSteamApp> {
    let steam_apps = get_installed_steam_apps();
    let app_id = match runtime {
        Runtime::SteamScout => "1070560",
        Runtime::SteamSoldier => "1391110",
        Runtime::SteamSniper => "1628350",
        _ => panic!("{} is not a steam runtime", runtime)
    };
    if let Some(app) = steam_apps.get(app_id) {
        return Some(app.clone());
    }
    None
}

pub fn get_steam_compat_tools_path() -> PathBuf {
    PathBuf::from(get_steam_path().unwrap()).join("compatibilitytools.d")
}

pub fn read_cmd_from_manifest(manifest_path: &PathBuf) -> anyhow::Result<String> {
    let tool_manifest = read_vdf(&manifest_path)?;

    Ok(tool_manifest["manifest"]
        .get("commandline")
        .and_then(|v| v.as_str())
        .unwrap_or_else(|| "")
        .to_string()
        .strip_prefix("/").unwrap_or_else(|| "")
        .to_string())
}

pub fn parse_steam_compat_tool_from_app(app: InstalledSteamApp) -> anyhow::Result<CompatTool> {
    let cmd = read_cmd_from_manifest(&app.path.join("toolmanifest.vdf"))?;
    Ok(CompatTool {
        name: app.name.to_string(),
        dir_path: app.path.to_str().unwrap().to_string(),
        path: app.path.join(cmd.replace(" %verb%", ""))
            .to_str().unwrap().to_string()
    })
}

pub fn parse_steam_compat_tool(path: PathBuf) -> anyhow::Result<CompatTool> {
    let compat_tool_vdf = read_vdf(&path.join("compatibilitytool.vdf"))?;
    let compat_tool_data: &Entry = compat_tool_vdf["compatibilitytools"]
        .get("compat_tools")
        .and_then(|v| v.as_table())
        .and_then(|v| v.values().next())
        .unwrap();

    if let Some(osfrom) = compat_tool_data.get("from_oslist") {
        let osfrom_str = osfrom.as_str().unwrap_or_default().to_lowercase();
        if osfrom_str != "windows" {
            return Err(anyhow!("Not a windows to linux compat tool, skipping."))
        }
    }

    let compat_tool_dir_path = path.join(
        compat_tool_data.get("install_path")
            .and_then(|v| v.as_str())
            .unwrap_or_else(|| "")
    ).canonicalize()?;

    let compat_tool_display_name = compat_tool_data.get("display_name").unwrap().as_str().unwrap_or_else(|| "Borken");

    let cmd = read_cmd_from_manifest(&path.join("toolmanifest.vdf"))?;

    Ok(CompatTool {
        name: compat_tool_display_name.to_string(),
        dir_path: compat_tool_dir_path.to_str().unwrap().to_string(),
        path: compat_tool_dir_path.join(cmd.replace(" %verb%", ""))
            .to_str().unwrap().to_string()
    })
}

pub fn list_steam_compat_tools() -> Vec<CompatTool> {
    let steam_compat_tools_path = get_steam_compat_tools_path();

    let mut results = vec![];
    if let Ok(entries) = fs::read_dir(steam_compat_tools_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            if let Ok(steam_compat_tool) = parse_steam_compat_tool(path) {
                results.push(steam_compat_tool);
            }
        }
    }

    let steam_apps = get_installed_steam_apps();
    STEAM_VALID_COMPAT_APPIDS.iter().for_each(|app_id| {
        if let Some(app) = steam_apps.get(&app_id.to_string()) {
            if let Ok(steam_compat_tool) = parse_steam_compat_tool_from_app(app.clone()) {
                results.push(steam_compat_tool);
            }
        }
    });

    results
}

pub fn get_steam_env_app_id() -> Result<String, env::VarError> {
    env::var("STEAM_COMPAT_APP_ID")
}

pub fn create_compatibility_tool_vdf(compat_tool_name: &str, display_name: &str) -> String {
    let mut string_builder = String::new();

    string_builder.push_str(r#""compatibilitytools"
{
  "compat_tools"
  {
    ""#);
    string_builder.push_str(compat_tool_name);
    string_builder.push_str(r#"" // Internal name of this tool
    {
      // Can register this tool with Steam in two ways:
      //
      // - The tool can be placed as a subdirectory in compatibilitytools.d, in which case this
      //   should be '.'
      //
      // - This manifest can be placed directly in compatibilitytools.d, in which case this should
      //   be the relative or absolute path to the tool's dist directory.
      "install_path" "."

      // For this template, we're going to substitute the display_name key in here, e.g.:
      "display_name" ""#);
    string_builder.push_str(display_name);
    string_builder.push_str(r#""

      "from_oslist"  "windows"
      "to_oslist"    "linux"
    }
  }
}
"#);
    string_builder
}
