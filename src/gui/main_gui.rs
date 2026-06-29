use crate::gui::globals::global_init_trait::ComponentInitExt;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use device_query::{DeviceQuery, DeviceState, Keycode};
use serde_yaml::{Value};
use slint::{ComponentHandle, ToSharedString, VecModel, Weak};
use tokio::time::{interval};
use crate::{AppConf, EnvVar, EnvVarsSettings, GUIGPUVendor, HardRefresh, MainGUI};
use crate::compatibility_tools::compat_tool::{get_compat_tool_from_config, CompatTool, CompatToolType};
use crate::compatibility_tools::compat_tools_list::{CompatToolsList};
use crate::config::global_config::GlobalConfig;
use crate::config::serialized_config_utils::{SerializedConfig};
use crate::gpu_tools::gpu::{GPU};
use crate::gpu_tools::gpu_list::GPUList;
use crate::gui::dialog::show_message_dialog;
use crate::gui::globals::init_hard_refresh::WindowForceRefresh;
use crate::install::install::{install_or_update};
use crate::steam::steam::get_steam_env_app_id;
use crate::utils::find_index::{FindIndex};
use crate::utils::slint_utils::{ClonableModel, WeakUtils};

fn init_gui_with_conf(window: &MainGUI, shared_config: Arc<Mutex<SerializedConfig>>) {
    // Compat tool list
    let full_compat_tools = CompatToolsList::get();
    let compat_tools = full_compat_tools
        .iter()
        .filter(|ct| ct.compat_type != CompatToolType::SimpleSteamWrapper)
        .collect::<Vec<&CompatTool>>();
    let compat_tools_model: ClonableModel<&CompatTool> = ClonableModel::new(compat_tools);
    window.set_compat_tools(compat_tools_model.to_model_rc(|ct| ct.name.clone()));

    let initial_compat_tool = get_compat_tool_from_config();
    let initial_compat_tool_index = initial_compat_tool.clone()
        .and_then(|cct| compat_tools_model.find_index(|ct| ct.name == cct.name))
        .unwrap_or(0);

    let initial_runtime_name = initial_compat_tool.clone()
        .and_then(|ct| ct.required_runtime)
        .and_then(|v| Some(v.name))
        .unwrap_or("None".to_string());

    let initial_compat_tool_type = initial_compat_tool
        .and_then(|ct| Some(ct.compat_type))
        .unwrap_or(CompatToolType::Proton);

    // GPU List
    let gpus = GPUList::get();
    let gpu_model: ClonableModel<GPU> = ClonableModel::new(gpus);
    window.set_gpus(gpu_model.to_model_rc(|gpu| gpu.full_name.clone()));
    let gpu_from_conf = GPU::from_config();
    let initial_gpu_index = gpu_model.find_index(|g| {
        &gpu_from_conf.as_formatted_id() == &g.as_formatted_id()
    }).unwrap_or(0);

    // Workaround: https://github.com/slint-ui/slint/issues/7632
    let _ = window.as_weak().upgrade_in_event_loop(move |w| {
        w.set_current_runtime(initial_runtime_name.into());
        w.set_compat_tool_type(initial_compat_tool_type.to_shared_string());
        w.set_selected_compat_tool_index(initial_compat_tool_index);
        w.set_selected_gpu_index(initial_gpu_index);
        w.invoke_on_gpu_update(initial_gpu_index);
    });

    // Env vars
    let is_editing_default = window.global::<AppConf>().get_editing_defaults();
    window.global::<EnvVarsSettings>().set_env_vars(
        Rc::new({
            let shared_serialized_conf = shared_config.clone();
            let borrowed_serialized_conf = shared_serialized_conf.lock().unwrap();
            borrowed_serialized_conf
                .get_app_value("custom_env_vars", is_editing_default)
                .and_then(|v| v.as_mapping())
                .unwrap()
                .iter()
                .map(|(k, v)| EnvVar {
                    key: k.as_str().unwrap().into(),
                    value: v.as_str().unwrap().into()
                })
                .collect::<VecModel<_>>()
        }).into()
    );
}

async fn wait_for_key_loop(window: Weak<MainGUI>, shared_config: Arc<Mutex<SerializedConfig>>) {
    let mut loop_interval = interval(Duration::from_millis(50));
    loop {
        loop_interval.tick().await;

        let device_state = DeviceState::new();
        let keys: Vec<Keycode> = device_state.get_keys();
        if let Some(pressed_key) = keys.get(0) {
            if *pressed_key != Keycode::Escape {
                shared_config.lock().unwrap()
                    .set_global_value_from_string("general.gui_trigger_key", &pressed_key.to_string());
            }
            let shared_config = shared_config.clone();
            let _ = window.upgrade_in_event_loop(move |w| {
                w.set_setting_key({
                    let borrowed = shared_config.lock().unwrap();
                    borrowed.get_global_value_from_string("general.gui_trigger_key")
                });
                w.set_is_setting_key(false);
            });
            break;
        }
    }
}

pub fn show_main_gui() -> bool {
    let _ = slint::set_xdg_app_id("fr.Skydust.SimpleSteamWrapper");
    let window = MainGUI::new().expect("Couldn't create window");

    let shared_config: Arc<Mutex<SerializedConfig>> = Arc::new(Mutex::new(SerializedConfig::from_global_config()));

    window.init_global::<AppConf>(shared_config.clone());
    window.init_global::<HardRefresh>({
        let weak_window = window.as_weak();
        let shared_config = shared_config.clone();
        Box::new(move || {
            let shared_config = shared_config.clone();
            weak_window.upgrade_and_run(|w| {
                init_gui_with_conf(&w, shared_config)
            })
        })
    });
    window.init_global::<EnvVarsSettings>(shared_config.clone());

    let version = env!("CARGO_PKG_VERSION");
    window.set_app_version(version.into());

    let steam_app_id = get_steam_env_app_id().unwrap_or("".to_string());
    window.set_game_app_id((&steam_app_id).into());
    if steam_app_id.is_empty() {
        window.global::<AppConf>().set_editing_defaults(true);
    }

    window.on_on_compat_tool_change({
        let weak_window = window.as_weak();
        let shared_config = shared_config.clone();

        move |_| {
            shared_config.lock().unwrap().update_global_config();
            let compat_tool = get_compat_tool_from_config();
            if let Some(compat_tool) = compat_tool {
                weak_window.upgrade_and_run(|w| {
                    w.set_current_runtime(compat_tool.required_runtime
                        .and_then(|r| Some(r.name))
                        .unwrap_or("None".to_string()).into());
                    w.set_compat_tool_type(compat_tool.compat_type.to_shared_string());
                })
            }
        }
    });

    window.on_on_gpu_update({
        let weak_window = window.as_weak();
        let shared_config = shared_config.clone();
        move |idx| {
            // Resetting nvidia vars
            weak_window.upgrade_and_run(|w| {
                let mut borrow_config = shared_config.lock().unwrap();

                let gpus = GPUList::get();
                let selected = gpus.get(idx as usize).unwrap().as_formatted_id().into();

                let is_editing_defaults = w.global::<AppConf>().get_editing_defaults();
                borrow_config.set_app_value("selected_gpu", selected, is_editing_defaults);

                borrow_config.update_global_config();
                let gpu_from_conf = GPU::from_config();
                let is_nvidia = gpu_from_conf.is_nvidia();

                if !is_nvidia {
                    borrow_config.set_app_value("enabled_tweaks.proton_nvapi", Value::from(false), is_editing_defaults);
                    borrow_config.set_app_value("enabled_tweaks.proton_nvapi_vkreflex", Value::from(false), is_editing_defaults);
                }

                // Lazy vendor_name for now
                let vendor_name: GUIGPUVendor = {
                    if is_nvidia {
                        GUIGPUVendor::Nvidia
                    } else {
                        GUIGPUVendor::Others
                    }
                };

                w.set_current_gpu_vendor(vendor_name.into());
            });
        }
    });

    window.on_update_wrapper(|| {
        install_or_update();
        show_message_dialog("Successfully updated wrapper");
    });

    window.on_show_download_runner({
        let shared_config = shared_config.clone();
        let weak_window = window.as_weak();
        move || {
            let weak_window = weak_window.clone();
            shared_config.lock().unwrap().update_global_config();
            crate::gui::dl_manager_gui::show_gui(weak_window);
        }
    });

    window.on_reset_to_defaults({
        let shared_config = shared_config.clone();
        let weak_window = window.as_weak();
        move || {
            let window_default = weak_window.upgrade().unwrap();
            shared_config.lock().unwrap()
                .reset_serialized_opts_to_defaults(window_default.global::<AppConf>().get_editing_defaults());
            window_default.force_refresh();
        }
    });

    window.on_enable_setting_key({
        let shared_config = shared_config.clone();
        let weak_window = window.as_weak();
        move || {
            let shared_config = shared_config.clone();
            let weak_window = weak_window.clone();
            tokio::spawn(wait_for_key_loop(weak_window, shared_config));
        }
    });

    window.on_show_prefix_options({
        let shared_config = shared_config.clone();
        move || {
            shared_config.lock().unwrap().update_global_config();
            crate::gui::prefix_gui::show_gui();
        }
    });

    let should_continue = Rc::new(RefCell::new(false));
    window.on_save_and_continue({
        let weak_window = window.as_weak();
        let shared_config = shared_config.clone();
        let should_continue = should_continue.clone();
        move || {
            *should_continue.borrow_mut() = true;
            weak_window.upgrade_and_run(|w| {
                w.global::<EnvVarsSettings>().invoke_save_env_vars();
                shared_config.lock().unwrap().update_global_config();
                GlobalConfig::save();
                w.hide().unwrap();
                slint::quit_event_loop().expect("Unable to stop event loop");
            })
        }
    });

    init_gui_with_conf(&window, shared_config);

    let _ = window.run().unwrap();

    *should_continue.borrow()
}
