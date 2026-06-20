use std::process::Command;
use tweaks_macro::tweak;
use crate::tweak_collector::PreparedCommand;

#[tweak(name = "gamemode", priority=75)]
pub fn run(_: &mut Command, prepared_command: &mut PreparedCommand) {
    prepared_command.command_prefixes.insert(0, "gamemoderun".to_string());
}
