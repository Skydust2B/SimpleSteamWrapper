use log::debug;
use serde_yaml::Value;
use slint::SharedString;
use crate::config::config::Config;
use crate::config::config_loader::{get_steam_app_id, LOADED_CONFIG};

pub struct SerializedConfig {
    serialized_config: Value
}

pub fn value_to_shared_string(value: Option<&Value>) -> SharedString {
    match value {
        Some(Value::String(s)) => SharedString::from(s.as_str()),
        Some(Value::Number(n)) => SharedString::from(n.to_string()),
        Some(Value::Bool(b))   => SharedString::from(b.to_string()),
        _ => SharedString::default(),
    }
}

/// This function might be my downfall
fn parse_guess(val: String) -> Value {
    if let Ok(b) = val.parse::<bool>() { Value::Bool(b) }
    else if let Ok(i) = val.parse::<i64>() { Value::Number(i.into()) }
    else if let Ok(f) = val.parse::<f64>() {
        Value::Number(serde_yaml::Number::from(f))
    } else {
        Value::String(val.to_string())
    }
}

impl SerializedConfig {
    pub fn from_global_config() -> Self {
        Self {
            serialized_config: serde_yaml::to_value(&LOADED_CONFIG.get_config()).unwrap()
        }
    }

    /// Traverse a `serde_yaml::Value` by a dotted key.
    pub fn get_value(&self, key: &str) -> Option<&Value> {
        key.split('.')
            .fold(Some(&self.serialized_config), |acc, part| acc.and_then(|v| v.get(part)))
    }

    pub fn get_mutable_value(&mut self, key: &str) -> Option<&mut Value> {
        key.split('.')
            .fold(Some(&mut self.serialized_config), |acc, part| acc.and_then(|v| v.get_mut(part)))
    }

    /// Traverse a `serde_yaml::Value` by a dotted key and set it to a new value.
    pub fn set_value(
        &mut self,
        key: &str,
        val: Value
    ) {
        *self.get_mutable_value(key)
            .expect(&format!("The key doesn't seem to exist: {}", &key)) = val;
    }

    pub fn update_global_config(&self) {
        let conf = &self.serialized_config;
        let updated_conf: Config = serde_yaml::from_value(conf.clone())
            .expect("Failed to deserialize config after update");
        LOADED_CONFIG.set_config(updated_conf);
    }

    fn app_conf_exists(&self, app_id: &str) -> bool {
        (&self.serialized_config)
            .get("apps").unwrap()
            .as_mapping().unwrap()
            .get(Value::String(app_id.to_string()))
            .is_some()
    }

    pub fn get_app_value(&self, key: &str, is_editing_defaults: bool) -> Option<&Value> {
        let steam_app_id = get_steam_app_id();

        let conf_path = if !is_editing_defaults
            && steam_app_id.is_ok()
            && self.app_conf_exists(&steam_app_id.clone().unwrap())
        {
            format!("apps.{}.{}", &steam_app_id.unwrap(), key)
        } else {
            format!("defaults.{}", key)
        };

        debug!("get_opt: \"{}\"", conf_path);
        self.get_value(&conf_path)
    }

    pub fn get_app_value_as_string(&self, key: &str, is_editing_defaults: bool) -> SharedString {
        value_to_shared_string(self.get_app_value(&key, is_editing_defaults))
    }

    fn get_or_create_app_conf_to_edit(&mut self, app_id: &str) -> &mut Value {
        let mutable_serialized_conf = &mut self.serialized_config;

        let defaults_clone = mutable_serialized_conf.get("defaults").cloned().unwrap();
        let app_id_entry = Value::String(app_id.to_string());
        let apps = mutable_serialized_conf
            .get_mut("apps").unwrap()
            .as_mapping_mut().unwrap();

        apps.entry(app_id_entry)
            .or_insert(defaults_clone)
    }

    /// Handles defaults vs per-app config.
    pub fn set_app_value(&mut self, key: &str, val: Value, is_editing_defaults: bool) {
        let steam_app_id = get_steam_app_id();

        let conf_path = if !is_editing_defaults && steam_app_id.is_ok() {
            let steam_app_id = steam_app_id.unwrap();
            self.get_or_create_app_conf_to_edit(&steam_app_id);
            format!("apps.{}.{}", steam_app_id, key)
        } else {
            format!("defaults.{}", key)
        };

        debug!("set_opt: {:?} -> {:?} (is_editing_default: {})", &key, &val, is_editing_defaults);

        // Enabled_tweaks is a partial map, that should probably change later
        if key.contains("enabled_tweaks.") {
            let last = key.split(".").last().unwrap();
            let parent_key = conf_path.replace(&format!(".{}", last), "");
            let mutable_value = self.get_mutable_value(&parent_key)
                .expect(&format!("Missing base key enabled_tweaks for {}", &parent_key));

            mutable_value.as_mapping_mut().unwrap()
                .insert(Value::from(last), val);
            return;
        }
        self.set_value(&conf_path, val);
    }

    /// Handles defaults vs per-app config.
    pub fn set_app_value_from_string(&mut self, key: &str, val: &str, is_editing_defaults: bool) {
        self.set_app_value(&key, parse_guess(val.to_string()), is_editing_defaults);
    }

    pub fn reset_serialized_opts_to_defaults(
        &mut self,
        is_editing_defaults: bool
    ) {
        let mutable_serialized_conf = &mut self.serialized_config;
        let steam_app_id = get_steam_app_id();
        if is_editing_defaults {
            let new_defaults: Value = serde_yaml::to_value(Config::new().defaults).unwrap();
            *mutable_serialized_conf.get_mut("defaults").unwrap() = new_defaults;
        } else if steam_app_id.is_ok() {
            if let Some(map) = mutable_serialized_conf.get_mut("apps").unwrap().as_mapping_mut() {
                map.remove(&Value::String(steam_app_id.unwrap().to_string())).unwrap_or_default();
            }
        }
    }
}
