use std::{fs};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use directories::ProjectDirs;
use log::{error, info};
use once_cell::sync::Lazy;
use serde_yaml::Value;
use crate::config::config::{Config, Options};
use crate::gui::dialog::show_message_dialog;
use crate::steam::steam::get_steam_env_app_id;

#[derive(Clone)]
pub(crate) struct GlobalConfig {
    state: Arc<Mutex<Config>>,
}

static LOADED_CONFIG: Lazy<GlobalConfig> = Lazy::new(GlobalConfig::new);

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

impl GlobalConfig {
    fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(Config::new())),
        }
    }

    pub(crate) fn set(new_state: Config) {
        let mut state = LOADED_CONFIG.state.lock().unwrap();
        *state = new_state;
    }

    pub(crate) fn get() -> Config {
        let state = LOADED_CONFIG.state.lock().unwrap();
        state.clone()
    }

    pub(crate) fn get_app_options() -> Options {
        let state = LOADED_CONFIG.state.lock().unwrap();
        let steam_app_id = get_steam_env_app_id().unwrap_or_default();
        state.apps.get(&steam_app_id).unwrap_or(&state.defaults).clone()
    }

    /// Returns the standard config path for the current platform
    fn get_path() -> PathBuf {
        if let Some(proj_dirs) = ProjectDirs::from("fr", "Skydust", "SimpleSteamWrapper") {
            let config_dir = proj_dirs.config_dir();
            fs::create_dir_all(config_dir).expect("Failed to create config directory");
            config_dir.join("config.yaml")
        } else {
            panic!("Could not determine config directory for this platform");
        }
    }

    /// Serializes and saves the configuration on disk
    pub(crate) fn save() {
        info!("Saving config");
        let state = Self::get();
        let path = Self::get_path();
        let yaml = serde_yaml::to_string(&state).expect("Failed to serialize config");
        fs::write(&path, yaml).expect("Failed to write config file");
    }

    /// Merges defaults inside the config file
    fn parse_config_file(content: String) -> anyhow::Result<Config> {
        let parsed_config = Config::new();
        let yaml_value = serde_yaml::from_str::<Value>(&content)?;

        // Convert default config to Value
        let default_value = serde_yaml::to_value(&parsed_config)?;
        // Merge YAML into defaults
        let mut merged = merge_yaml(default_value.clone(), yaml_value);

        let default_app_config = default_value.get("defaults").unwrap();
        let apps_mapping = merged.get_mut("apps")
            .and_then(|v| v.as_mapping_mut())
            .unwrap();

        // Merge defaults onto each apps
        apps_mapping.clone()
            .iter_mut()
            .for_each(|(k, v)| {
                apps_mapping.insert(k.clone(), merge_yaml(default_app_config.clone(), v.clone()));
            });

        let deserialized_value = serde_yaml::from_value::<Config>(merged);
        if let Ok(cfg) = deserialized_value {
            Ok(cfg)
        } else {
            let error = deserialized_value.err()
                .and_then(|v| Some(v.to_string()))
                .unwrap_or(String::new());
            Err(anyhow::Error::msg(format!("Failed to deserialize config: {}", error)))
        }
    }

    /// Loads the YAML config from disk
    pub(crate) fn load() -> Config {
        let path = Self::get_path();
        info!("Reading configuration from: {}", path.display());

        if !path.exists() {
            // Save default config
            info!("No configuration file, writing...");
            let yaml = serde_yaml::to_string(&Config::new()).unwrap();
            fs::write(&path, yaml).expect("Failed to write config");
        }

        let content = fs::read_to_string(&path).unwrap_or_default();

        let parsed_cfg = Self::parse_config_file(content);
        if let Ok(cfg) = parsed_cfg {
            GlobalConfig::set(cfg.clone());
            cfg
        } else {
            let error = parsed_cfg.err()
                .and_then(|v| Some(v.to_string()))
                .unwrap_or(String::new());

            show_message_dialog(&format!("Couldn't read the config file, some values might be wrong\n\n{}", error));
            slint::run_event_loop().unwrap();
            std::process::exit(1);
        }
    }
}
