use std::process::Command;
use tweaks_macro::tweak;

#[tweak(name = "mangohud", priority=100)]
pub fn run(_: &mut Command, prepared_command: &mut Vec<String>) {
    prepared_command.insert(0, "mangohud".to_string());
}
