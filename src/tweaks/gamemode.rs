use std::process::Command;
use tweaks_macro::tweak;

#[tweak(name = "gamemode")]
pub fn run(_: &mut Command, prepared_command: &mut Vec<String>) {
    prepared_command.insert(0, "gamemoderun".to_string());
}
