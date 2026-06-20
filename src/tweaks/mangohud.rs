use std::process::Command;
use tweaks_macro::tweak;
use crate::tweak_collector::PreparedCommand;

#[tweak(name = "mangohud", priority=100)]
pub fn run(_: &mut Command, prepared_command: &mut PreparedCommand) {
    prepared_command.command_prefixes.insert(0, "mangohud".to_string());
}
