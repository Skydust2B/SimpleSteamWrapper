use std::process::Command;
use tweaks_macro::tweak;
use crate::config::config_loader::LOADED_CONFIG;

#[tweak(name = "custom_env_vars")]
pub fn run(process: &mut Command, _: &mut Vec<String>) {
    let custom_env_vars = LOADED_CONFIG.get_app_options().custom_env_vars;
    custom_env_vars.iter().for_each(|(key, val)| {
        process.env(key, val);
    });
}
