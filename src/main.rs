extern crate core;

mod tweaks;
mod config;
mod runner;
mod install;
pub(crate) mod tweak;
mod compatibility_tools;
mod gpu_tools;
mod command_helpers;
mod gui;
mod dl_manager;

use std::{env};
use tracing_subscriber::EnvFilter;
use device_query::{DeviceQuery, DeviceState, Keycode};
use log::info;
use crate::config::config_loader::load_config;
use crate::gui::main_gui::show_gui;
use crate::runner::game_process_wrapper::run_game_process;
use crate::install::install::check_install;

slint::include_modules!();

#[tokio::main]
async fn main() {
    let rust_log = env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string());
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new(format!("{},tracing=warn", rust_log)))
        .init();

    info!("RUST_LOG: {}", rust_log);
    load_config();

    let is_in_steam_env = env::var("STEAM_COMPAT_APP_ID").and(Ok(true)).unwrap_or(false);
    if !is_in_steam_env {
        info!("Outside steam, running GUI");
        check_install();
        show_gui();
        return;
    }

    let device_state = DeviceState::new();
    let keys: Vec<Keycode> = device_state.get_keys();
    if keys.contains(&Keycode::LShift) { // hold Shift to show GUI
        show_gui();
    }
    run_game_process();
}
