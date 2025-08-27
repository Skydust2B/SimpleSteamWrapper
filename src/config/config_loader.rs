use std::{env, fs};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use directories::ProjectDirs;
use log::info;
use once_cell::sync::Lazy;
use serde_yaml::Value;
use slint::SharedString;
use crate::config::config::{Config, Options};

#[derive(Clone)]
pub(crate) struct ConfigState {
    state: Arc<Mutex<Config>>,
}

impl ConfigState {
    fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(Config::new())),
        }
    }

    pub(crate) fn set_config(&self, new_state: Config) {
        let mut state = self.state.lock().unwrap();
        *state = new_state;
    }

    pub(crate) fn get_config(&self) -> Config {
        let state = self.state.lock().unwrap();
        state.clone()
    }

    pub(crate) fn get_app_options(&self) -> Options {
        let state = self.state.lock().unwrap();
        let steam_app_id = get_steam_app_id().unwrap_or_default();
        state.apps.get(&steam_app_id).unwrap_or(&state.defaults).clone()
    }

    pub(crate) fn save(&self) {
        info!("Saving config");
        let state = self.get_config();
        let path = get_config_path();
        let yaml = serde_yaml::to_string(&state).expect("Failed to serialize config");
        fs::write(&path, yaml).expect("Failed to write config file");
    }
}

pub static LOADED_CONFIG: Lazy<ConfigState> = Lazy::new(ConfigState::new);

pub fn get_steam_app_id() -> Result<String, env::VarError> {
    env::var("STEAM_COMPAT_APP_ID")
}

/// Returns the standard config path for the current platform
fn get_config_path() -> PathBuf {
    if let Some(proj_dirs) = ProjectDirs::from("fr", "Skydust", "SimpleSteamWrapper") {
        let config_dir = proj_dirs.config_dir();
        fs::create_dir_all(config_dir).expect("Failed to create config directory");
        config_dir.join("config.yaml")
    } else {
        panic!("Could not determine config directory for this platform");
    }
}

/// Loads the YAML config from disk, or returns default if missing
pub fn load_config() -> Config {
    let path = get_config_path();
    info!("Reading configuration from: {}", path.display());

    let mut parsed_config = Config::new();
    let mut had_error = false;
    if path.exists() {
        let content = fs::read_to_string(&path).unwrap_or_default();

        if let Ok(yaml_value) = serde_yaml::from_str::<Value>(&content) {
            // Convert default config to Value
            let default_value = serde_yaml::to_value(&parsed_config).unwrap();

            // Merge YAML into defaults
            let merged = merge_yaml(default_value, yaml_value);

            // Deserialize back into Config
            if let Ok(cfg) = serde_yaml::from_value::<Config>(merged) {
                parsed_config = cfg;
            } else {
                had_error = true;
                eprintln!("Warning: failed to deserialize merged config. Using defaults for invalid fields.");
            }
        } else {
            had_error = true;
            eprintln!("Warning: failed to parse config.yaml. Using defaults for invalid fields.");
        }
    }
    if !path.exists() || had_error {
        // Save default config
        info!("Invalid configuration file, rewriting...");
        let yaml = serde_yaml::to_string(&parsed_config).unwrap();
        fs::write(&path, yaml).expect("Failed to write default config");
    };

    LOADED_CONFIG.set_config(parsed_config.clone());

    parsed_config
}

// Recursive YAML merge function
fn merge_yaml(mut base: Value, override_: Value) -> Value {
    match (&mut base, override_) {
        (Value::Mapping(base_map), Value::Mapping(override_map)) => {
            for (k, v) in override_map {
                if let Some(existing) = base_map.get_mut(&k) {
                    *existing = merge_yaml(existing.clone(), v);
                } else {
                    base_map.insert(k, v);
                }
            }
            base
        }
        (_, override_val) => override_val, // Override scalar or array values
    }
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
    *options_to_edit.get_mut(last).unwrap() = parse_guess(val.to_string());
}

pub fn reset_serialized_opts_to_defaults(
    conf: &mut Value,
    is_editing_defaults: bool
) {
    let steam_app_id = get_steam_app_id();
    if is_editing_defaults {
        let new_defaults: Value = serde_yaml::to_value(Config::new().defaults).unwrap();
        *conf.get_mut("defaults").unwrap() = new_defaults;
    } else {
        if let Some(map) = conf.get_mut("apps").unwrap().as_mapping_mut() {
            map.remove(&Value::String(steam_app_id.unwrap().to_string()));
        }
    }
}
