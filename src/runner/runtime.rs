use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString, VariantArray};
use crate::runner::game_process_wrapper::{get_run_verb, RunVerb};
use crate::steam::steam::get_steam_runtime_app;

#[derive(Debug, EnumString, VariantArray, Serialize, Deserialize, Clone, PartialEq, Display)]
pub enum Runtime {
    None,
    SteamScout,
    SteamSoldier,
    SteamSniper
}

impl Runtime {
    pub fn get_runtime_entrypoint(&self) -> String {
        let runtime_verb = get_run_verb().unwrap_or(RunVerb::Run);

        let runtime_path = match self {
            Runtime::SteamScout => {
                Some(get_steam_runtime_app(self.clone())
                    .expect("Unable to get runtime entrypoint")
                    .path.join("_v2-entry-point"))
            }
            Runtime::SteamSoldier => {
                Some(get_steam_runtime_app(self.clone())
                    .expect("Unable to get runtime entrypoint")
                    .path.join("_v2-entry-point"))
            }
            Runtime::SteamSniper => {
                Some(get_steam_runtime_app(self.clone())
                    .expect("Unable to get runtime entrypoint")
                    .path.join("_v2-entry-point"))
            },
            Runtime::None => None,
        };

        if let Some(runtime_path) = runtime_path {
            return format!("{} --verb={} --", runtime_path.to_str().unwrap(), runtime_verb);
        }
        "".to_string()
    }
}
