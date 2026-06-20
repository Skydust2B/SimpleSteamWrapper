use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString, VariantArray};
use crate::runner::game_process_wrapper::{get_run_verb, RunVerb};
use crate::steam::steam::get_steam_runtime_app;

#[derive(Debug, EnumString, VariantArray, Serialize, Deserialize, Clone, PartialEq, Display)]
#[strum(serialize_all = "lowercase")]
pub enum Runtime {
    SteamScout,
    SteamSoldier,
    SteamSniper
}

impl Runtime {
    pub fn get_runtime_entrypoint(&self) -> String {
        let runtime_verb = get_run_verb().unwrap_or(RunVerb::Run);

        let runtime_path = match self {
            Runtime::SteamScout => {
                get_steam_runtime_app(self.clone())
                    .expect("Unable to get runtime entrypoint")
                    .path.join("_v2-entry-point")
            }
            Runtime::SteamSoldier => {
                get_steam_runtime_app(self.clone())
                    .expect("Unable to get runtime entrypoint")
                    .path.join("_v2-entry-point")
            }
            Runtime::SteamSniper => {
                get_steam_runtime_app(self.clone())
                    .expect("Unable to get runtime entrypoint")
                    .path.join("_v2-entry-point")
            }
        };

        format!("{} --verb={} --", runtime_path.to_str().unwrap(), runtime_verb)
    }
}
