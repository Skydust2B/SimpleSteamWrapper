use std::{env, io};
use std::path::PathBuf;
use std::process::{ExitStatus, Stdio};
use tokio::process::{Command};
use log::{info, warn};
use crate::command_helpers::{find_terminal_emulator, to_quoted_string};
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

    env_vars.push(("WINEPREFIX".to_string(), PathBuf::from(data_path).join("pfx").to_str().unwrap().to_string()));

    let compat_tool = get_compat_tool_from_config();
    let wine_binary_path = PathBuf::from(compat_tool.dir_path).join("files/bin/wine");
    env_vars.push(("WINE".to_string(), wine_binary_path.to_str().unwrap().to_string()));

    let game_data_path = env::var("STEAM_COMPAT_INSTALL_PATH").expect("STEAM_COMPAT_INSTALL_PATH must be set");
    env_vars.push(("PWD".to_string(), game_data_path));

    env_vars.push(("WINEDEBUG".to_string(),"-all".to_string()));

    env_vars
}

pub async fn run_wiretricks_in_prefix() -> io::Result<ExitStatus> {
    let data_path = PathBuf::from(env::var("STEAM_COMPAT_DATA_PATH").expect("STEAM_COMPAT_DATA_PATH must be set"));
    let compat_tool = get_compat_tool_from_config();

    info!("Running winetricks with {} at {}", compat_tool.name, data_path.display());
    let mut process = Command::new("winetricks");
    process
        .arg("--gui")
        .envs(get_wine_variables())
        .env("LD_PRELOAD", "")
        .env("LD_LIBRARY_PATH", "")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status().await
}

pub async fn reset_prefix() -> io::Result<ExitStatus> {
    let data_path = PathBuf::from(env::var("STEAM_COMPAT_DATA_PATH").expect("STEAM_COMPAT_DATA_PATH must be set"));
    if data_path.exists() {
        info!("Removing prefix at {}", data_path.display());
        tokio::fs::remove_dir_all(&data_path).await.expect("Unable to remove prefix");
    }
    let compat_tool = get_compat_tool_from_config();

    info!("Recreating prefix with {} at {}", compat_tool.name, data_path.display());
    let mut process = Command::new(compat_tool.path);

    process
        .args(&["run", "/bin/true"])
        .env("STEAM_COMPAT_DATA_PATH", data_path.to_str().unwrap())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status().await
}

pub async fn run_in_prefix(executable: PathBuf, in_terminal: bool) -> io::Result<ExitStatus> {
    let compat_tool = get_compat_tool_from_config();
    let wine_binary_path = PathBuf::from(compat_tool.dir_path).join("files/bin/wine");

    let mut process = if in_terminal {
        find_terminal_emulator().map(|terminal| {
            let mut cmd = Command::new(terminal);
            cmd.arg("-e");
            cmd
        }).unwrap_or_else(|| {
            info!("Unable to find terminal emulator, falling back to sh");
            let mut cmd = Command::new("sh");
            cmd.arg("-c");
            cmd
        })
    } else {
        let mut cmd = Command::new("sh");
        cmd.arg("-c");
        cmd
    };

    process.envs(get_wine_variables())
        .env("LD_PRELOAD", "")
        .env("LD_LIBRARY_PATH", "")
        .env("WINEDEBUG", "")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .stdin(Stdio::inherit());

    let mut cmd = Vec::new();

    cmd.push(wine_binary_path.to_str().unwrap().to_string());
    cmd.push(executable.to_str().unwrap().to_string());

    let joined_cmd = to_quoted_string(cmd);
    info!("Running cmd in prefix: {}", joined_cmd);

    process
        .arg(joined_cmd);

    process.status().await
}
