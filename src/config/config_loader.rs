use std::{env, fs};
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
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
