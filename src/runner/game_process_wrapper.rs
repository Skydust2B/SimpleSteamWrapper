use std::{env};
use std::process::{Command, Stdio};
use std::str::FromStr;
use log::info;
use strum_macros::{Display, EnumString, VariantArray};
use crate::utils::command_utils::to_quoted_string;
use crate::compatibility_tools::app_prefix::AppPrefix;
use crate::config::config::{Config};
use crate::config::global_config::{GlobalConfig};
use crate::compatibility_tools::compat_tool::{CompatTool};
use crate::tweak_collector::{list_tweaks, PreparedCommand, Tweak};

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

    let mut prepared_command: PreparedCommand = PreparedCommand {
        command_prefixes: Vec::new(),
        arguments: env::args().skip(2).collect::<Vec<String>>()
    };

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
            info!("Running tweak \"{}\" (Priority: {})", tweak.name, tweak.priority);
            (tweak.execute)(&mut process, &mut prepared_command);
        }
    });

    AppPrefix::from_env(); // Used to validate proton env

    let mut wrapper_prepared_command = Vec::new();

    let run_verb = get_run_verb().unwrap_or(RunVerb::Run);

    if run_verb == RunVerb::Waitforexitandrun && prepared_command.command_prefixes.len() > 0{
        wrapper_prepared_command.extend_from_slice(prepared_command.command_prefixes.as_slice());
    }

    let runtime = app_config.clone().runtime;
    wrapper_prepared_command.extend_from_slice(runtime.get_runtime_entrypoint().as_slice());

    wrapper_prepared_command.extend_from_slice(&[
        compat_tool.path.to_string(),
        run_verb.to_string()
    ]);

    wrapper_prepared_command.extend_from_slice(prepared_command.arguments.as_slice());

    let wrapper_command_as_str = to_quoted_string(wrapper_prepared_command);

    info!("Running command: {}", wrapper_command_as_str);

    info!("With environment variables:");
    process.get_envs().for_each(|(key, val)|  {
        info!("{}={}", key.to_str().unwrap_or_default(), val.unwrap_or_default().to_str().unwrap_or_default())
    });

    let status = process
        .arg(wrapper_command_as_str)
        .status()
        .expect("Failed to spawn child");

    Some(status)
}
