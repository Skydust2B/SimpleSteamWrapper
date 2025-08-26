use std::{env};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use log::info;
use crate::config::config::{Config};
use crate::config::config_loader::LOADED_CONFIG;
use crate::runner::compat_tools_wrapper::{get_compat_tool_from_config, get_steam_path};
use crate::tweaks::tweak::{list_tweaks, Tweak};

fn to_quoted_string(args: Vec<String>) -> String {
    format!("\"{}\"", args.join("\" \""))
}

pub fn run_game_process() {
    if let Some(_) = env::args().nth(1) {
        let mut prepared_command: Vec<String> = Vec::new();
        
        let mut process = Command::new("sh");
        process
            .arg("-c")
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .stdin(Stdio::inherit());

        let tweaks: Vec<_> = list_tweaks();
        let mut iterator_tweaks = tweaks.iter().collect::<Vec<&&Tweak>>();
        iterator_tweaks.sort_by_key(|v| v.priority);

        let config: Config = LOADED_CONFIG.get_config();

        let steam_app_id = env::var("STEAM_COMPAT_APP_ID").expect("STEAM_COMPAT_APP_ID not set");
        let app_config = config.apps.get(&steam_app_id).unwrap_or(&config.defaults);

        iterator_tweaks.iter().for_each(|tweak| {
            if *app_config.enabled_tweaks.get(tweak.name).unwrap_or(&false) {
                info!("Running tweak \"{}\"", tweak.name);
                (tweak.execute)(&mut process, &mut prepared_command);
            }
        });

        let steam_runtime_path = get_steam_path().unwrap().join("steamapps/common/SteamLinuxRuntime_sniper");
        if !steam_runtime_path.exists() {
            panic!("Could not find steam runtime");
        }
        let steam_runtime_run_path = PathBuf::from(steam_runtime_path).join("_v2-entry-point");
        env::var("STEAM_COMPAT_DATA_PATH").expect("STEAM_COMPAT_DATA_PATH must be set");

        let compat_tool = get_compat_tool_from_config();
        let passed_arguments = env::args().skip(2).collect::<Vec<String>>();

        let mut wrapper_prepared_command = String::new();
        let mut run_verb = "run";
        if env::args().nth(1).unwrap() == "waitforexitandrun" {
            run_verb = "waitforexitandrun";

            if prepared_command.len() > 0 {
                wrapper_prepared_command = format!("{} ", to_quoted_string(prepared_command));
            }
        }

        wrapper_prepared_command = format!("{}\"{}\" --verb={} -- \"{}\" {} {}",
            wrapper_prepared_command,
            steam_runtime_run_path.to_str().unwrap(),
            run_verb,
            compat_tool.path.to_string().replace(" %verb%", ""),
            run_verb,
            to_quoted_string(passed_arguments));

        info!("Running command: {}", wrapper_prepared_command);

        info!("With environment variables:");
        process.get_envs().for_each(|(key, val)|  {
            info!("{}={}", key.to_str().unwrap_or_default(), val.unwrap_or_default().to_str().unwrap_or_default())
        });

        let status = process
            .arg(wrapper_prepared_command)
            .status()
            .expect("Failed to spawn child");

        info!("Exit status: {}", status);
        std::process::exit(status.code().unwrap_or(1));
    }
}
