use std::{env, fs};
use std::path::PathBuf;
use std::string::ToString;
use anyhow::{anyhow, Context};
use log::warn;
use vdf_reader::entry::{Entry, Table};
use crate::compatibility_tools::compat_tool::{CompatTool};
use crate::steam::installed_steam_apps::{get_installed_steam_apps, InstalledSteamApp};
use crate::runner::runtime::Runtime;

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

pub fn parse_runtime_from_appid(appid: String) -> anyhow::Result<Runtime> {
    let app: InstalledSteamApp = get_installed_steam_apps().get(appid.as_str())
        .ok_or(anyhow!("Can't find steam app {}", appid))?.clone();

    let manifest = read_from_manifest(&app.path.join("toolmanifest.vdf"))?;

    if manifest.compatmanager_layer_name != Some("container-runtime".to_string()) {
        return Err(anyhow!("Not a container runtime"));
    }

    let splitted_cmd = manifest.cmd.split(" ").map(|val| val.to_string()).collect::<Vec<String>>();

    Ok(Runtime {
        path: app.path,
        cmdline: splitted_cmd,
        name: app.name.clone()
    })
}

pub fn get_steam_compat_tools_path() -> PathBuf {
    PathBuf::from(get_steam_path().unwrap()).join("compatibilitytools.d")
}

struct SteamToolManifest {
    cmd: String,
    required_tool_appid: Option<String>,
    compatmanager_layer_name: Option<String>
}

fn read_from_manifest(manifest_path: &PathBuf) -> anyhow::Result<SteamToolManifest> {
    let tool_manifest = read_vdf(&manifest_path)?;

    let cmd = tool_manifest["manifest"]
        .get("commandline")
        .and_then(|v| v.as_str())
        .unwrap_or_else(|| "")
        .to_string()
        .strip_prefix("/").unwrap_or_else(|| "")
        .to_string();

    let required_tool_appid = tool_manifest["manifest"]
        .get("require_tool_appid")
        .and_then(|v| v.as_str())
        .and_then(|v| Some(v.to_string()));

    let compatmanager_layer_name = tool_manifest["manifest"]
        .get("compatmanager_layer_name")
        .and_then(|v| v.as_str())
        .and_then(|v| Some(v.to_string()));

    Ok(SteamToolManifest{
        cmd,
        required_tool_appid,
        compatmanager_layer_name
    })
}

pub fn parse_steam_compat_tool_from_app(app: InstalledSteamApp) -> anyhow::Result<CompatTool> {
    let manifest = read_from_manifest(&app.path.join("toolmanifest.vdf"))?;

    if manifest.compatmanager_layer_name != Some("proton".to_string()) {
        return Err(anyhow!("Not a proton compat tool"));
    }

    let required_app = manifest.required_tool_appid.and_then(|appid| Some(parse_runtime_from_appid(appid))).transpose()?;

    Ok(CompatTool {
        name: app.name.to_string(),
        dir_path: app.path,
        cmd_line: manifest.cmd.split(" ").map(|v| v.to_string()).collect(),
        required_runtime: required_app
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

    let manifest = read_from_manifest(&path.join("toolmanifest.vdf"))?;
    let required_app = manifest.required_tool_appid
        .and_then(|appid| Some(parse_runtime_from_appid(appid.clone())
            .inspect_err(|err| warn!("Couldn't parse runtime from appid {}: {}", appid, err))
            )).transpose()?;

    Ok(CompatTool {
        name: compat_tool_display_name.to_string(),
        dir_path: compat_tool_dir_path,
        cmd_line: manifest.cmd.split(" ").map(|v| v.to_string()).collect(),
        required_runtime: required_app
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
    steam_apps.iter().for_each(|(_, app)| {
        let compat_tool = parse_steam_compat_tool_from_app(app.clone());
        if let Ok(compat_tool) = compat_tool {
            results.push(compat_tool);
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
