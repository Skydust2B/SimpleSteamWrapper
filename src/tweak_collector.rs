use std::process::Command;
use inventory::collect;

#[derive(Debug)]
pub struct PreparedCommand {
    pub command_prefixes: Vec<String>, // Before entrypoint
    pub arguments: Vec<String>
}

#[derive(Debug)]
pub struct Tweak {
    pub name: &'static str,
    pub priority: i32,
    pub execute: fn(process: &mut Command, prepared_command: &mut PreparedCommand)
}

collect!(Tweak);

pub fn list_tweaks() -> Vec<&'static Tweak> {
    inventory::iter::<Tweak>().collect()
}
