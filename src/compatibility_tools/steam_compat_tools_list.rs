use std::sync::{Arc, Mutex};
use log::debug;
use once_cell::sync::Lazy;
use crate::compatibility_tools::compat_tool::CompatTool;
use crate::compatibility_tools::steam::list_steam_compat_tools;

#[derive(Clone)]
pub(crate) struct SteamCompatToolsList {
    state: Arc<Mutex<Vec<CompatTool>>>,
}

impl SteamCompatToolsList {
    fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn refresh_list_internal(&self) {
        let mut state = self.state.lock().unwrap();
        let compat_tools = list_steam_compat_tools();
        debug!("Found {} steam compat tools", compat_tools.len());
        *state = compat_tools;
    }

    fn get_list_internal(&self) -> Vec<CompatTool> {
        let state = self.state.lock().unwrap();
        state.clone()
    }

    pub(crate) fn refresh_list() {
        LOADED_STEAM_COMPAT_TOOLS_LIST.refresh_list_internal();
    }

    pub(crate) fn get_list() -> Vec<CompatTool> {
        let mut list = LOADED_STEAM_COMPAT_TOOLS_LIST.get_list_internal();
        if list.is_empty() {
            LOADED_STEAM_COMPAT_TOOLS_LIST.refresh_list_internal();
            list = LOADED_STEAM_COMPAT_TOOLS_LIST.get_list_internal();
        }
        list
    }
}

static LOADED_STEAM_COMPAT_TOOLS_LIST: Lazy<SteamCompatToolsList> = Lazy::new(SteamCompatToolsList::new);
