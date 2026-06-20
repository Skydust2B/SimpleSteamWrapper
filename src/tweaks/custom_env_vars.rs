use std::process::Command;
use tweaks_macro::tweak;
use crate::config::global_config::GlobalConfig;
use crate::tweak_collector::PreparedCommand;

#[tweak(name = "custom_env_vars")]
pub fn run(process: &mut Command, _: &mut PreparedCommand) {
    let custom_env_vars = GlobalConfig::get_app_options().custom_env_vars;
    custom_env_vars.iter().for_each(|(key, val)| {
        process.env(key, val);
    });
}
