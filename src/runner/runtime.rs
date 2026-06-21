use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use crate::runner::game_process_wrapper::RunVerb;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Runtime {
    pub path: PathBuf,
    pub cmdline: Vec<String>,
    pub name: String
}

impl Runtime {
    pub fn get_exec_path(&self) -> PathBuf {
        self.path.join(self.cmdline[0].as_str())
    }

    pub fn get_full_command(&self, verb: RunVerb) -> Vec<String> {
        let exec_path = self.get_exec_path();
        let mut command = vec![exec_path.to_str().expect("Unable to parse the full command").to_string()];
        command.extend_from_slice(&self.cmdline[1..]);
        command.iter().map(|v| v.replace("%verb%", &verb.to_string())).collect()
    }
}
