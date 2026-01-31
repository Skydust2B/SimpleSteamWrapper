use std::{env, fs};
use log::{info, warn};
use crate::compatibility_tools::steam::{create_compatibility_tool_vdf, get_steam_compat_tools_path};
use crate::compatibility_tools::steam_compat_tools_list::SteamCompatToolsList;
use crate::install::install_modal::show_install_modal;

const TOOL_MANIFEST_VDF: &str = r#""manifest"
{
  "commandline" "/SimpleSteamWrapper run"
  "commandline_waitforexitandrun" "/SimpleSteamWrapper waitforexitandrun"
}
"#;

const SIMPLE_STEAM_WRAPPER_NAME: &str = "SimpleSteamWrapper";

pub fn install_or_update() {
    let simple_steam_wrapper_path = get_steam_compat_tools_path().join(SIMPLE_STEAM_WRAPPER_NAME);

    if simple_steam_wrapper_path.exists() {
        warn!("Simple Steam Wrapper is installed, removing to update");
        warn!("{}", simple_steam_wrapper_path.display());
        fs::remove_dir_all(&simple_steam_wrapper_path).expect("Unable to remove Steam Wrapper");
    }

    let current_exe_path = env::current_exe().unwrap();
    fs::create_dir(&simple_steam_wrapper_path).expect("Unable to create the directory");
    info!("Executable path: {}", current_exe_path.display());
    fs::copy(env::current_exe().unwrap(), simple_steam_wrapper_path.join(SIMPLE_STEAM_WRAPPER_NAME)).expect("Unable to copy the executable");
    info!("Writing vfd files...");
    let compat_tool_vdf_path = simple_steam_wrapper_path.join("compatibilitytool.vdf");

    // We need to keep Proton in the string for steam cloud sync to work
    let compat_vdf_file = create_compatibility_tool_vdf("Proton-SimpleSteamWrapper", "SimpleSteamWrapper");
    fs::write(compat_tool_vdf_path, compat_vdf_file).expect("Unable to write compatibilitytool.vdf");
    let tool_manifest_vdf_path = simple_steam_wrapper_path.join("toolmanifest.vdf");
    fs::write(tool_manifest_vdf_path, TOOL_MANIFEST_VDF).expect("Unable to write toolmanifest.vdf");

    info!("Successfully installed, restarting steam might be necessary");
}

pub fn check_install() {
    let steam_compat_tools = SteamCompatToolsList::get_list();
    if !steam_compat_tools.iter().any(|ct| ct.name == SIMPLE_STEAM_WRAPPER_NAME) {
        show_install_modal();
    }
}
