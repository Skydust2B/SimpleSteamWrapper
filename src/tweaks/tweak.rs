use std::process::Command;
use inventory::collect;

#[derive(Debug)]
pub struct Tweak {
    pub name: &'static str,
    pub priority: i32,
    pub execute: fn(process: &mut Command, prepared_command: &mut Vec<String>)
}

collect!(Tweak);

pub fn list_tweaks() -> Vec<&'static Tweak> {
    inventory::iter::<Tweak>().collect()
}
