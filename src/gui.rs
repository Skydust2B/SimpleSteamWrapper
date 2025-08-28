use std::cell::RefCell;
use std::rc::Rc;
use serde_yaml::Value;
use slint::{ComponentHandle, ModelRc, SharedString, VecModel};
use crate::config::config::Config;
use crate::config::config_loader::{get_serialized_config_value, get_steam_app_id, reset_serialized_opts_to_defaults, set_serialized_config_value, LOADED_CONFIG};
use crate::gpu::{get_gpu_from_config, list_all_gpus};
use crate::{AppConf, MainGUI};
use crate::install::install::install_or_update;
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

    let steam_app_id = get_steam_app_id().unwrap_or("".to_string());
    window.set_game_app_id(SharedString::from(&steam_app_id));
    if steam_app_id.is_empty() {
        window.set_editing_defaults(true);
    }

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

    window.on_get_combobox_gpu_id(|index| {
        let gpus_for_ids = list_all_gpus();
        SharedString::from(gpus_for_ids.get(index as usize).unwrap().as_formatted_id())
    });

    let gpu_from_conf = get_gpu_from_config();
    let initial_gpu_index = find_index(&gpus, |g| {
        &gpu_from_conf.as_formatted_id() == &g.as_formatted_id()
    }).unwrap();

    // Workaround: https://github.com/slint-ui/slint/issues/7632
    let _ = slint::invoke_from_event_loop({
        let weak_window = window.as_weak();
        move || {
            if let Some(window) = weak_window.upgrade() {
                window.set_selected_compat_tool_index(initial_compat_tool_index);
                window.set_selected_gpu_index(initial_gpu_index);
            }
        }
    });

    let serialized_conf: Rc<RefCell<Value>> =
        Rc::new(RefCell::new(serde_yaml::to_value(&LOADED_CONFIG.get_config()).unwrap()));

    // Getter
    window.global::<AppConf>().on_get_opt({
        let shared_serialized_conf = Rc::clone(&serialized_conf);
        let weak_window = window.as_weak();
        move |key| {
            let is_editing_defaults = weak_window.upgrade().unwrap().get_editing_defaults();
            let get_conf = shared_serialized_conf.borrow(); // borrow here, inside the closure
            get_serialized_config_value(&get_conf, &key, is_editing_defaults)
        }
    });

    // Setter
    window.global::<AppConf>().on_set_opt({
        let shared_serialized_conf = Rc::clone(&serialized_conf);
        let weak_window = window.as_weak();
        move |key, val| {
            let is_editing_defaults = weak_window.upgrade().unwrap().get_editing_defaults();
            let mut set_conf = shared_serialized_conf.borrow_mut();
            set_serialized_config_value(&mut set_conf, &key, &val, is_editing_defaults);
        }
    });

    window.on_force_reload({
        let weak_window = window.as_weak();
        move || {
            let window_reload = weak_window.upgrade().unwrap();
            window_reload.set_reload(true);
            window_reload.window().request_redraw();
            let _ = slint::invoke_from_event_loop({
                let nw = window_reload.as_weak();
                move || {
                    if let Some(w) = nw.upgrade() {
                        w.set_reload(false);
                        w.window().request_redraw();
                    }
                }
            });
        }
    });

    window.on_update_wrapper(|| {
        install_or_update();
    });

    window.on_reset_to_defaults({
        let shared_serialized_conf = Rc::clone(&serialized_conf);
        let weak_window = window.as_weak();
        move || {
            let window_default = weak_window.upgrade().unwrap();
            reset_serialized_opts_to_defaults(&mut shared_serialized_conf.borrow_mut(), window_default.get_editing_defaults());
            window_default.set_selected_compat_tool_index(initial_compat_tool_index);
            window_default.set_selected_gpu_index(initial_gpu_index);
            window_default.invoke_force_reload();
        }
    });

    let _ = window.run().unwrap();

    let updated_conf: Config = serde_yaml::from_value((*serialized_conf.borrow()).clone()).unwrap();
    LOADED_CONFIG.set_config(updated_conf);
    LOADED_CONFIG.save();
}
