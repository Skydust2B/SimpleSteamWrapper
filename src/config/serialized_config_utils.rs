use std::cell::RefCell;
use std::rc::Rc;
use serde_yaml::Value;
use slint::SharedString;
use crate::config::config::Config;
use crate::config::config_loader::{get_steam_app_id, LOADED_CONFIG};

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


/// Deserializes and saves the config in memory
pub fn update_config_from_serialized(serialized_conf: &Rc<RefCell<Value>>) {
    let updated_conf: Config = serde_yaml::from_value((*serialized_conf.borrow()).clone()).unwrap();
    LOADED_CONFIG.set_config(updated_conf);
}


/// Traverse a `serde_yaml::Value` by a dotted key and return it as a `SharedString`.
pub fn get_serialized_config_value(conf: &Value, key: &str, is_editing_defaults: bool) -> SharedString {
    let steam_app_id = get_steam_app_id().unwrap_or_default();

    let apps_conf = conf
        .get("apps").unwrap()
        .as_mapping().unwrap()
        .get(Value::String(steam_app_id.clone()));

    let options_to_read = if !is_editing_defaults && apps_conf.is_some() {
        apps_conf.unwrap()
    } else {
        conf.get("defaults").unwrap()
    };

    let val = key
        .split('.')
        .fold(Some(options_to_read), |acc, part| acc.and_then(|v| v.get(part)));

    match val {
        Some(Value::String(s)) => SharedString::from(s.as_str()),
        Some(Value::Number(n)) => SharedString::from(n.to_string()),
        Some(Value::Bool(b))   => SharedString::from(b.to_string()),
        _ => SharedString::default(),
    }
}

/// Traverse a `serde_yaml::Value` by a dotted key and set it to a new string value.
/// Handles defaults vs per-app config.
pub fn set_serialized_config_value(
    conf: &mut Value,
    key: &str,
    val: &str,
    is_editing_defaults: bool,
) {
    let defaults_clone = conf.get("defaults").cloned().unwrap();

    let steam_app_id = get_steam_app_id();

    let mut options_to_edit = if !is_editing_defaults && steam_app_id.is_ok() {
        let apps = conf
            .get_mut("apps").unwrap()
            .as_mapping_mut().unwrap();

        apps.entry(Value::String(steam_app_id.unwrap()))
            .or_insert(defaults_clone)
    } else {
        conf.get_mut("defaults").unwrap()
    };

    // Traverse down to target field
    let parts: Vec<&str> = key.split('.').collect();
    for part in &parts[..parts.len() - 1] {
        options_to_edit = options_to_edit.get_mut(part).unwrap();
    }

    // Update the final field
    let last = parts.last().unwrap();

    if let Some(map) = options_to_edit.as_mapping_mut() {
        map.insert(Value::String(last.to_string()), parse_guess(val.to_string()));
    } else {
        *options_to_edit.get_mut(last).expect(format!("The key doesn't seem to exist: {}", &key).as_str()) = parse_guess(val.to_string());
    }
}

pub fn reset_serialized_opts_to_defaults(
    conf: &mut Value,
    is_editing_defaults: bool
) {
    let steam_app_id = get_steam_app_id();
    if is_editing_defaults {
        let new_defaults: Value = serde_yaml::to_value(Config::new().defaults).unwrap();
        *conf.get_mut("defaults").unwrap() = new_defaults;
    } else if steam_app_id.is_ok() {
        if let Some(map) = conf.get_mut("apps").unwrap().as_mapping_mut() {
            map.remove(&Value::String(steam_app_id.unwrap().to_string())).unwrap_or_default();
        }
    }
}
