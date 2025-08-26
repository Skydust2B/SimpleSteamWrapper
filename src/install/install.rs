use std::{env, fs};
use log::{info, warn};
use crate::install::install_modal::show_install_modal;
use crate::runner::compat_tools_wrapper::{get_steam_compat_tools_path, list_steam_compat_tools};

const COMPATIBILITY_TOOL_VDF: &str = r#""compatibilitytools"
{
  "compat_tools"
  {
    "SimpleSteamWrapper" // Internal name of this tool
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
      "display_name" "SimpleSteamWrapper"

      "from_oslist"  "windows"
      "to_oslist"    "linux"
    }
  }
}
"#;

const TOOL_MANIFEST_VDF: &str = r#""manifest"
{
  "version" "2"
  "commandline" "/SimpleSteamWrapper %verb%"
  "require_tool_appid" "1628350"
  "use_sessions" "1"
  "compatmanager_layer_name" "proton"
}
"#;

const SIMPLE_STEAM_WRAPPER_NAME: &str = "SimpleSteamWrapper";

pub fn install_or_update() {
    let simple_steam_wrapper_path = get_steam_compat_tools_path().join(SIMPLE_STEAM_WRAPPER_NAME);

    if simple_steam_wrapper_path.exists() {
        warn!("Simple Steam Wrapper is installed, removing to update");
        fs::remove_dir_all(&simple_steam_wrapper_path).expect("Unable to remove Steam Wrapper");
    }

    fs::create_dir(&simple_steam_wrapper_path).expect("Unable to create the directory");
    fs::copy(env::current_exe().unwrap(), simple_steam_wrapper_path.join(SIMPLE_STEAM_WRAPPER_NAME)).expect("Unable to copy the executable");

    let compat_tool_vdf_path = simple_steam_wrapper_path.join("compatibilitytool.vdf");
    fs::write(compat_tool_vdf_path, COMPATIBILITY_TOOL_VDF).expect("Unable to write compatibilitytool.vdf");
    let tool_manifest_vdf_path = simple_steam_wrapper_path.join("toolmanifest.vdf");
    fs::write(tool_manifest_vdf_path, TOOL_MANIFEST_VDF).expect("Unable to write toolmanifest.vdf");

    info!("Successfully installed, restarting steam might be necessary");
}

pub fn check_install() {
    let steam_compat_tools = list_steam_compat_tools();
    if !steam_compat_tools.iter().any(|ct| ct.name == SIMPLE_STEAM_WRAPPER_NAME) {
        show_install_modal();
    }
}
