extern crate core;

mod tweaks;
mod config;
mod runner;
mod install;
pub(crate) mod tweak_collector;
mod compatibility_tools;
mod gpu_tools;
mod command_helpers;
mod gui;
mod dl_manager;
mod steam;
mod utils;

use std::{env};
use std::str::FromStr;
use tracing_subscriber::EnvFilter;
use device_query::{DeviceQuery, DeviceState, Keycode};
use log::info;
use crate::compatibility_tools::compat_tool::get_compat_tool_from_config;
use crate::config::global_config::{GlobalConfig};
use crate::gui::dialog::show_message_dialog;
use crate::gui::main_gui::show_gui;
use crate::runner::game_process_wrapper::run_game_process;
use crate::install::install::check_install;

slint::include_modules!();

#[tokio::main]
async fn main() {
    let rust_log = env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new(format!("{},tracing=warn,h2=warn,winit=warn,reqwest=warn,sctk=warn,hyper_util=warn,rustls_platform_verifier=warn", rust_log)))
        .init();

    info!("RUST_LOG: {}", rust_log);
    GlobalConfig::load();

    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    let is_in_steam_env = env::var("STEAM_COMPAT_APP_ID").and(Ok(true)).unwrap_or(false);
    if !is_in_steam_env {
        info!("Outside steam, running GUI");
        check_install();
        show_gui();
        return;
    }

    let device_state = DeviceState::new();
    let keys: Vec<Keycode> = device_state.get_keys();
    let config_key_code = GlobalConfig::get().general.gui_trigger_key;
    let parsed_key_code = &Keycode::from_str(&config_key_code).expect("Failed to parse keycode");
    if keys.contains(&parsed_key_code) { // hold Shift to show GUI
        show_gui();
    }

    if let Some(cfg_compat_tool) = get_compat_tool_from_config() {
        run_game_process(cfg_compat_tool);
    } else {
        show_message_dialog("No compatibility tools configured! This will exit");
    }
}
