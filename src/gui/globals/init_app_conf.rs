use std::sync::{Arc, Mutex};
use slint::{ComponentHandle};
use crate::AppConf;
use crate::config::serialized_config_utils::SerializedConfig;
use crate::gui::globals::global_init_trait::GlobalInitializer;

impl<T> GlobalInitializer<T> for AppConf<'_>
where
    T: ComponentHandle,
    for<'a> AppConf<'a>: slint::Global<'a, T>,
{
    type Ctx = Arc<Mutex<SerializedConfig>>;

    fn init_global(component: &T, shared_config: Arc<Mutex<SerializedConfig>>) {
        let app_conf_globals = component.global::<AppConf>();

        // Getter
        app_conf_globals.on_get_opt({
            let shared_serialized_conf = shared_config.clone();
            move |key, is_editing_defaults| {
                shared_serialized_conf.lock().unwrap()
                    .get_app_value_as_string(&key, is_editing_defaults)
            }
        });

        // Setter
        app_conf_globals.on_set_opt({
            let shared_serialized_conf = shared_config.clone();
            move |key, val, is_editing_defaults| {
                shared_serialized_conf.lock().unwrap()
                    .set_app_value_from_string(&key, &val, is_editing_defaults);
            }
        });

        app_conf_globals.on_update_global_config({
            let shared_serialized_conf = shared_config.clone();
            move || {
                shared_serialized_conf.lock().unwrap()
                    .update_global_config()
            }
        });
    }
}
