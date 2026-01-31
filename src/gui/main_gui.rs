use std::cell::RefCell;
use std::rc::Rc;
use log::{debug};
use serde_yaml::{Mapping, Value};
use slint::{ComponentHandle, Model, ModelRc, SharedString, VecModel};
use crate::config::config_loader::{get_steam_app_id, LOADED_CONFIG};
use crate::{AppConf, MainGUI};
use crate::compatibility_tools::compat_tool::{get_compat_tool_from_config};
use crate::compatibility_tools::steam_compat_tools_list::SteamCompatToolsList;
use crate::config::serialized_config_utils::{SerializedConfig};
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

fn load_values_from_conf(window: &MainGUI, shared_config: Rc<RefCell<SerializedConfig>>) {
    let compat_tools = SteamCompatToolsList::get_list();
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
    let _ = window.as_weak().upgrade_in_event_loop(move |w| {
        w.set_selected_compat_tool_index(initial_compat_tool_index);
        w.set_selected_gpu_index(initial_gpu_index);
    });

    let is_editing_default = window.get_editing_defaults();
    window.set_env_vars(
        Rc::new(VecModel::from({
            let shared_serialized_conf = Rc::clone(&shared_config);
            let borrowed_serialized_conf = shared_serialized_conf.borrow();
            borrowed_serialized_conf
                .get_app_value("custom_env_vars", is_editing_default)
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

fn save_custom_values_into_conf(window: &MainGUI, shared_config: Rc<RefCell<SerializedConfig>>) {
    let is_editing_default = window.get_editing_defaults();

    let new_opts =
        window.get_env_vars()
            .iter().fold(
            Mapping::default(),
            |mut acc, (key, val)|{
            acc.insert(Value::from(key.to_string()), Value::from(val.to_string()));
            acc
        });

    shared_config.borrow_mut().set_app_value(
        "custom_env_vars",
        new_opts.into(),
        is_editing_default
    )
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

    let serialized_conf: Rc<RefCell<SerializedConfig>> =
        Rc::new(RefCell::new(SerializedConfig::from_global_config()));

    // Getter
    window.global::<AppConf>().on_get_opt({
        let shared_serialized_conf = Rc::clone(&serialized_conf);
        let weak_window = window.as_weak();
        move |key| {
            let is_editing_defaults = weak_window.upgrade().unwrap().get_editing_defaults();
            shared_serialized_conf
                .borrow()
                .get_app_value_as_string(&key, is_editing_defaults)
        }
    });

    // Setter
    window.global::<AppConf>().on_set_opt({
        let shared_serialized_conf = Rc::clone(&serialized_conf);
        let weak_window = window.as_weak();
        move |key, val| {
            let is_editing_defaults = weak_window.upgrade().unwrap().get_editing_defaults();
            shared_serialized_conf
                .borrow_mut()
                .set_app_value_from_string(&key, &val, is_editing_defaults);
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

            let _ = window_reload.as_weak().upgrade_in_event_loop(|window| {
                window.set_reload(false);
                window.window().request_redraw();
            });
        }
    });

    window.on_update_wrapper(|| {
        install_or_update();
        show_message_dialog("Successfully updated wrapper");
    });

    window.on_show_download_runner({
    let shared_serialized_conf = Rc::clone(&serialized_conf);
    move || {
        shared_serialized_conf.borrow().update_global_config();
        crate::gui::dl_manager_gui::show_gui();
    }});

    window.on_reset_to_defaults({
        let shared_serialized_conf = Rc::clone(&serialized_conf);
        let weak_window = window.as_weak();
        move || {
            let window_default = weak_window.upgrade().unwrap();
            shared_serialized_conf
                .borrow_mut()
                .reset_serialized_opts_to_defaults(window_default.get_editing_defaults());
            window_default.invoke_force_reload();
        }
    });

    window.on_save_custom_values({
        let shared_serialized_conf = Rc::clone(&serialized_conf);
        let weak_window = window.as_weak();

        move || {
            let window_save = weak_window.upgrade().unwrap();
            save_custom_values_into_conf(&window_save, shared_serialized_conf.clone());
            shared_serialized_conf.borrow().update_global_config();
        }
    });

    window.on_show_prefix_options({
        let shared_serialized_conf = Rc::clone(&serialized_conf);
        move || {
            shared_serialized_conf.borrow().update_global_config();
            crate::gui::prefix_gui::show_gui();
        }
    });

    let _ = window.run().unwrap();

    save_custom_values_into_conf(&window, serialized_conf.clone());

    serialized_conf.borrow().update_global_config();
    LOADED_CONFIG.save();
}
