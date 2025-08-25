use std::{env};
use std::process::{Command, Stdio};
use log::info;
use crate::config::config::{Config};
use crate::config::config_loader::LOADED_CONFIG;
use crate::tweaks::tweak::{list_tweaks, Tweak};

pub fn run_game_process() {
    if let Some(_) = env::args().nth(1) {
        let mut prepared_command: Vec<String> = env::args().skip(1).collect();

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

        let app_config = config.apps.get(&prepared_command[0]).unwrap_or(&config.defaults);

        iterator_tweaks.iter().for_each(|tweak| {
            if *app_config.enabled_tweaks.get(tweak.name).unwrap_or(&false) {
                info!("Running tweak \"{}\"", tweak.name);
                (tweak.execute)(&mut process, &mut prepared_command);
            }
        });

        info!("System env vars");
        env::vars().for_each(|(key, val)|  {
            info!("{}={}", key, val)
        });

        info!("Running command: {:?}", prepared_command);

        info!("With environment variables:");
        process.get_envs().for_each(|(key, val)|  {
            info!("{}={}", key.to_str().unwrap_or_default(), val.unwrap_or_default().to_str().unwrap_or_default())
        });

        let status = process
            .arg(prepared_command.join(" "))
            .status()
            .expect("Failed to spawn child");

        std::process::exit(status.code().unwrap_or(1));
    }
}
