use std::cell::RefCell;
use std::rc::Rc;
use log::{debug};
use serde_yaml::Value;
use slint::{ComponentHandle, Model, ModelRc, SharedString, VecModel};
use crate::config::config::Config;
use crate::config::config_loader::{get_serialized_config_value, get_steam_app_id, reset_serialized_opts_to_defaults, set_serialized_config_value, LOADED_CONFIG};
use crate::{AppConf, MainGUI};
use crate::compatibility_tools::compat_tools_wrapper::{get_compat_tool_from_config};
use crate::compatibility_tools::steam::list_steam_compat_tools;
use crate::gpu_tools::gpu::{get_gpu_from_config, list_all_gpus};
use crate::gui::dialog::show_message_dialog;
use crate::install::install::install_or_update;

fn find_index<T, F>(items: &[T], predicate: F) -> Option<i32>
where
    F: Fn(&T) -> bool,
{
    items.iter()
        .position(predicate)
        .and_then(|idx| i32::try_from(idx).ok())
}

fn load_values_from_conf(window: &MainGUI, shared_config: Rc<RefCell<Value>>) {
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
    let _ = slint::invoke_from_event_loop({
        let weak_window = window.as_weak();
        move || {
            if let Some(window) = weak_window.upgrade() {
                window.set_selected_compat_tool_index(initial_compat_tool_index);
                window.set_selected_gpu_index(initial_gpu_index);
            }
        }
    });

    window.set_env_vars(
        Rc::new(VecModel::from({
            let shared_serialized_conf = Rc::clone(&shared_config);
            let borrowed_serialized_conf = shared_serialized_conf.borrow();
            let steam_app_id = get_steam_app_id().unwrap_or("".to_string());
            borrowed_serialized_conf
                .get("apps").unwrap().get(steam_app_id)
                .unwrap_or_else(|| borrowed_serialized_conf.get("defaults").unwrap())
                .get("custom_env_vars")
                .unwrap()
                .as_mapping()
                .unwrap()
                .iter()
                .map(|(k, v)| (
                    SharedString::from(k.as_str().unwrap_or_default()),
                    SharedString::from(v.as_str().unwrap_or_default()),
                ))
                .collect::<Vec<_>>()
        })).into()
    );
}

fn save_custom_values_into_conf(window: &MainGUI, shared_config: Rc<RefCell<Value>>) {
    let is_editing_default = window.get_editing_defaults();

    let shared_serialized_conf = Rc::clone(&shared_config);
    let steam_app_id = get_steam_app_id().unwrap_or("".to_string());
    let mut borrowed_serialized_conf = shared_serialized_conf.borrow_mut();

    let mut app_opts = borrowed_serialized_conf
        .get_mut("apps").unwrap().get_mut(steam_app_id);

    if is_editing_default || app_opts.is_none() {
        app_opts = borrowed_serialized_conf.get_mut("defaults");
    }

    let mapping_opts = app_opts.unwrap()
        .get_mut("custom_env_vars").unwrap()
        .as_mapping_mut().unwrap();

    mapping_opts.clear();
    window.get_env_vars().iter().for_each(|(key, val)|{
        mapping_opts
            .insert(Value::from(key.to_string()), Value::from(val.to_string()));
    });
}

pub fn show_gui() {
    let _ = slint::set_xdg_app_id("fr.Skydust.SimpleSteamWrapper");

    let window = MainGUI::new().unwrap();

    let version = env!("CARGO_PKG_VERSION");
    window.set_app_version(SharedString::from(version));

    let steam_app_id = get_steam_app_id().unwrap_or("".to_string());
    window.set_game_app_id(SharedString::from(&steam_app_id));
    if steam_app_id.is_empty() {
        window.set_editing_defaults(true);
    }

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
            debug!("set_opt: {:?} -> {:?} (default: {})", &key, &val, is_editing_defaults);
            set_serialized_config_value(&mut set_conf, &key, &val, is_editing_defaults);
        }
    });

    window.on_get_combobox_gpu_id(|v| {
        let gpus = list_all_gpus();
        SharedString::from(gpus.get(v as usize).unwrap().as_formatted_id())
    });

    load_values_from_conf(&window, serialized_conf.clone());
    window.on_add_env_var({
        let weak_window = window.as_weak();
        move || {
            let env_window = weak_window.upgrade().unwrap();
            let env_vars = env_window.get_env_vars().iter().collect::<VecModel<(SharedString, SharedString)>>();
            env_vars.push((SharedString::new(), SharedString::new()));
            env_window.set_env_vars(ModelRc::from(Rc::new(env_vars)));
        }
    });

    window.on_remove_env_var({
        let weak_window = window.as_weak();
        move |val| {
            let env_window = weak_window.upgrade().unwrap();
            let env_vars = env_window.get_env_vars().iter().collect::<VecModel<(SharedString, SharedString)>>();
            env_vars.remove(val as usize);
            env_window.set_env_vars(ModelRc::from(Rc::new(env_vars)));
        }
    });

    window.on_force_reload({
        let shared_serialized_conf = Rc::clone(&serialized_conf);
        let weak_window = window.as_weak();
        move || {
            let window_reload = weak_window.upgrade().unwrap();
            window_reload.set_reload(true);
            window_reload.window().request_redraw();

            load_values_from_conf(&window_reload, shared_serialized_conf.clone());

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
        show_message_dialog("Successfully updated wrapper");
    });

    window.on_reset_to_defaults({
        let shared_serialized_conf = Rc::clone(&serialized_conf);
        let weak_window = window.as_weak();
        move || {
            let window_default = weak_window.upgrade().unwrap();
            reset_serialized_opts_to_defaults(&mut shared_serialized_conf.borrow_mut(), window_default.get_editing_defaults());
            window_default.invoke_force_reload();
        }
    });

    window.on_save_custom_values({
        let shared_serialized_conf = Rc::clone(&serialized_conf);
        let weak_window = window.as_weak();

        move || {
            let window_save = weak_window.upgrade().unwrap();
            save_custom_values_into_conf(&window_save, shared_serialized_conf.clone());
        }
    });

    window.on_show_prefix_options({
        let shared_serialized_conf = Rc::clone(&serialized_conf);
        move || {
            let updated_conf: Config = serde_yaml::from_value((*shared_serialized_conf.borrow()).clone()).unwrap();
            LOADED_CONFIG.set_config(updated_conf);
            crate::gui::prefix_gui::show_gui();
        }
    });

    let _ = window.run().unwrap();
    save_custom_values_into_conf(&window, serialized_conf.clone());

    let updated_conf: Config = serde_yaml::from_value((*serialized_conf.borrow()).clone()).unwrap();
    LOADED_CONFIG.set_config(updated_conf);
    LOADED_CONFIG.save();
}
