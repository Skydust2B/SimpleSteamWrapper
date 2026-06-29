use std::path::PathBuf;
use log::{info, warn};
use strum_macros::{Display, EnumString, VariantArray};
use crate::compatibility_tools::compat_tools_list::{CompatToolsList};
use crate::config::global_config::{GlobalConfig};
use crate::runner::game_process_wrapper::RunVerb;
use crate::runner::runtime::Runtime;

#[derive(Debug, EnumString, VariantArray, Clone, PartialEq, Display)]
#[strum(serialize_all = "kebab-case")]
pub enum CompatToolType {
    SimpleSteamWrapper,
    Proton,
    ScoutInContainer
}

#[derive(Debug, Clone)]
pub struct CompatTool {
    pub compat_type: CompatToolType,
    pub name: String,
    pub dir_path: PathBuf,
    pub cmd_line: Vec<String>,
    pub required_runtime: Option<Runtime>
}

impl CompatTool {
    pub fn find_wine_bin(&self) -> Option<PathBuf> {
        if self.compat_type != CompatToolType::Proton {
            return None;
        }
        let dir_path = PathBuf::from(self.clone().dir_path);
        let candidates = [
            dir_path.join("files/bin/wine"),
            dir_path.join("bin/wine"),
            dir_path.join("usr/bin/wine")
        ];

        candidates.into_iter().find(|p| p.exists())
    }

    pub fn get_exec_path(&self) -> PathBuf {
        self.dir_path.join(self.cmd_line[0].as_str())
    }

    pub fn get_full_command(&self, verb: RunVerb) -> Vec<String> {
        let exec_path = self.get_exec_path();
        let mut command = vec![exec_path.to_str().expect("Unable to parse the full command").to_string()];
        command.extend_from_slice(&self.cmd_line[1..]);
        command.iter().map(|v| v.replace("%verb%", &verb.to_string())).collect()
    }
}

pub fn get_compat_tool_from_config() -> Option<CompatTool> {
    let cfg = GlobalConfig::get_app_options();
    let all_ct = CompatToolsList::get();

    let retrieved_ct = all_ct.iter().find(|ct| cfg.compat_tool == ct.name);
    if cfg.compat_tool == "" {
        return None;
    }
    if retrieved_ct.is_none() {
        warn!("Unable to find selected compatibility tool {}", cfg.compat_tool);
        return None;
    }
    info!("Found selected compatibility tool {}", cfg.compat_tool);
    Some(retrieved_ct.unwrap().clone())
}
