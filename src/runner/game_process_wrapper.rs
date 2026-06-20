use std::{env};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::str::FromStr;
use log::info;
use strum_macros::{Display, EnumString, VariantArray};
use crate::utils::command_utils::to_quoted_string;
use crate::compatibility_tools::app_prefix::AppPrefix;
use crate::config::config::{Config};
use crate::config::global_config::{GlobalConfig};
use crate::compatibility_tools::compat_tool::{CompatTool};
use crate::gui::dialog::show_message_dialog;
use crate::steam::steam::get_steam_sniper_runtime;
use crate::tweak_collector::{list_tweaks, Tweak};

#[derive(Debug, EnumString, VariantArray, Clone, PartialEq, Display)]
#[strum(serialize_all = "lowercase")]
pub enum RunVerb {
    Run,
    Waitforexitandrun
}

pub fn get_run_verb() -> Option<RunVerb> {
    if let Some(verb) = env::args().nth(1) {
        if let Ok(parsed) = RunVerb::from_str(&verb) {
            return Some(parsed);
        }
    }
    None
}

pub fn run_game_process(compat_tool: CompatTool) -> Option<std::process::ExitStatus> {
    if env::args().nth(1).is_none() {
        return None;
    }
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

    let config: Config = GlobalConfig::get();

    let steam_app_id = env::var("STEAM_COMPAT_APP_ID").expect("STEAM_COMPAT_APP_ID not set");
    let app_config = config.apps.get(&steam_app_id).unwrap_or(&config.defaults);

    iterator_tweaks.iter().for_each(|tweak| {
        if *app_config.enabled_tweaks.get(tweak.name).unwrap_or(&false) {
            info!("Running tweak \"{}\"", tweak.name);
            (tweak.execute)(&mut process, &mut prepared_command);
        }
    });

    let steam_runtime = get_steam_sniper_runtime();
    if steam_runtime.is_none() {
        show_message_dialog("Couldn't find the steam sniper runtime, it is required to run proton games through steam.");
        return None;
    }
    let steam_runtime_run_path = PathBuf::from(steam_runtime.unwrap().path).join("_v2-entry-point");

    AppPrefix::from_env(); // Used to validate proton env

    let mut wrapper_prepared_command = String::new();

    let run_verb = get_run_verb().unwrap_or(RunVerb::Run);
    let passed_arguments = env::args().skip(2).collect::<Vec<String>>();

    if run_verb == RunVerb::Waitforexitandrun {
        if prepared_command.len() > 0 {
            wrapper_prepared_command = format!("{} ", to_quoted_string(prepared_command));
        }
    }

    wrapper_prepared_command = format!("{}\"{}\" --verb={} -- \"{}\" {} {}",
        wrapper_prepared_command,
        steam_runtime_run_path.to_str().unwrap(),
        run_verb.to_string(),
        compat_tool.path.to_string(),
        run_verb.to_string(),
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

    Some(status)
}
