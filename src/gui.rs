use std::cell::RefCell;
use std::rc::Rc;
use slint::{ComponentHandle, ModelRc, SharedString, VecModel};
use crate::config::config_loader::LOADED_CONFIG;
use crate::gpu::{get_gpu_from_config, list_all_gpus};
use crate::MainGUI;
use crate::runner::compat_tools_wrapper::{get_compat_tool_from_config, list_steam_compat_tools};

fn find_index<T, F>(items: &[T], predicate: F) -> Option<i32>
where
    F: Fn(&T) -> bool,
{
    items.iter()
        .position(predicate)
        .and_then(|idx| i32::try_from(idx).ok())
}

pub fn show_gui() {
    let window = MainGUI::new().unwrap();

    let compat_tools = list_steam_compat_tools();
    let compat_tools_names = compat_tools.iter().map(|ct| ct.name.clone().into()).collect::<Vec<SharedString>>();
    let model: ModelRc<SharedString> = Rc::new(VecModel::from(compat_tools_names)).into();
    window.set_compat_tools(model);

    let compat_tool_from_conf = get_compat_tool_from_config();
    let initial_compat_tool_index = find_index(&compat_tools, |ct| {
        ct.name == compat_tool_from_conf.name
    }).unwrap();

    let gpus = list_all_gpus();
    let gpu_names = gpus.iter().map(|g| g.full_name.clone().into()).collect::<Vec<SharedString>>();
    let model: ModelRc<SharedString> = Rc::new(VecModel::from(gpu_names)).into();
    window.set_gpus(model);

    let gpu_from_conf = get_gpu_from_config();
    let initial_gpu_index = find_index(&gpus, |g| {
        &gpu_from_conf.as_formatted_id() == &g.as_formatted_id()
    }).unwrap();

    // Workaround: https://github.com/slint-ui/slint/issues/7632
    let weak_window = window.as_weak();
    let _ = slint::invoke_from_event_loop(move || {
        if let Some(window) = weak_window.upgrade() {
            window.set_selected_compat_tool_index(initial_compat_tool_index);
            window.set_selected_gpu_index(initial_gpu_index);
        }
    });

    let conf = LOADED_CONFIG.get_config();
    let windows_tweak_states = Rc::new(RefCell::new(conf.defaults.enabled_tweaks));

    let states_clone = windows_tweak_states.clone();
    window.on_set_tweak_state(move |tweak_name, enabled| {
        states_clone.borrow_mut().insert(tweak_name.to_string(), enabled);
    });

    let states_clone = windows_tweak_states.clone();
    window.on_get_tweak_state(move |tweak_name| {
        *states_clone
            .borrow()
            .get(&tweak_name.to_string())
            .unwrap_or(&false)
    });

    let _ = window.run().unwrap();

    let mut mutable_conf = LOADED_CONFIG.get_config();

    mutable_conf.defaults.compat_tool = compat_tools.get(window.get_selected_compat_tool_index() as usize).unwrap().name.to_string();
    mutable_conf.defaults.selected_gpu = gpus.get(window.get_selected_gpu_index() as usize).unwrap().as_formatted_id();
    mutable_conf.defaults.enabled_tweaks = windows_tweak_states.borrow().clone();

    LOADED_CONFIG.set_config(mutable_conf);
    LOADED_CONFIG.save();
}
