use std::rc::Rc;
use std::sync::{Arc, Mutex};
use serde_yaml::{Mapping, Value};
use slint::{ComponentHandle, Model, VecModel};
use crate::{EnvVar, EnvVarsSettings};
use crate::config::serialized_config_utils::SerializedConfig;
use crate::gui::globals::global_init_trait::GlobalInitializer;

impl<T> GlobalInitializer<T> for EnvVarsSettings<'_>
where
    T: ComponentHandle + 'static,
    for<'a> EnvVarsSettings<'a>: slint::Global<'a, T>,
{
    type Ctx = Arc<Mutex<SerializedConfig>>;

    fn init_global(component: &T, shared_config: Arc<Mutex<SerializedConfig>>) {
        let env_vars_global = component.global::<EnvVarsSettings>();

        env_vars_global.on_update_serialized_env_vars({
            let shared_serialized_conf = shared_config.clone();
            move |env_vars, is_editing_defaults| {
                let new_opts =
                    env_vars.iter().fold(
                        Mapping::default(),
                        |mut acc, env_var| {
                            acc.insert(Value::from(env_var.key.to_string()), Value::from(env_var.value.to_string()));
                            acc
                        });

                shared_serialized_conf.lock().unwrap()
                    .set_app_value(
                    "custom_env_vars",
                    new_opts.into(),
                    is_editing_defaults
                );
            }
        });

        env_vars_global.on_add_env_var({
            let weak_window = component.as_weak();
            move || {
                let env_window = weak_window.upgrade().unwrap();
                let settings = env_window.global::<EnvVarsSettings>();
                let env_vars = settings.get_env_vars()
                    .iter()
                    .collect::<VecModel<EnvVar>>();
                env_vars.push(EnvVar::default());
                settings.set_env_vars(Rc::new(env_vars).into());
            }
        });

        env_vars_global.on_remove_env_var({
            let weak_window = component.as_weak();
            move |idx| {
                let env_window = weak_window.upgrade().unwrap();
                let settings = env_window.global::<EnvVarsSettings>();
                let env_vars = settings.get_env_vars()
                    .iter()
                    .collect::<VecModel<EnvVar>>();

                env_vars.remove(idx as usize);
                settings.set_env_vars(Rc::new(env_vars).into());
            }
        });
    }
}
