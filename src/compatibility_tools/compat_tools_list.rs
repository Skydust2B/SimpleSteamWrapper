use std::sync::{Arc, Mutex};
use log::debug;
use once_cell::sync::Lazy;
use crate::compatibility_tools::compat_tool::CompatTool;
use crate::steam::steam::list_steam_compat_tools;

#[derive(Clone)]
pub(crate) struct CompatToolsList {
    state: Arc<Mutex<Vec<CompatTool>>>,
}

impl CompatToolsList {
    fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn get_list_internal(&self) -> Vec<CompatTool> {
        let state = self.state.lock().unwrap();
        state.clone()
    }

    pub(crate) fn get() -> Vec<CompatTool> {
        let mut list = LOADED_COMPAT_TOOLS_LIST.get_list_internal();
        if list.is_empty() {
            Self::refresh();
            list = LOADED_COMPAT_TOOLS_LIST.get_list_internal();
        }
        list
    }

    pub(crate) fn refresh() {
        let mut state = LOADED_COMPAT_TOOLS_LIST.state.lock().unwrap();
        let compat_tools = list_steam_compat_tools();
        debug!("Found {} steam compat tools", compat_tools.len());
        *state = compat_tools;
    }
}

static LOADED_COMPAT_TOOLS_LIST: Lazy<CompatToolsList> = Lazy::new(CompatToolsList::new);
