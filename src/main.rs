extern crate core;

mod gui;
mod tweaks;
mod gpu;
mod config;
mod runner;
mod install;

use std::env;
use tracing_subscriber::EnvFilter;
use device_query::{DeviceQuery, DeviceState, Keycode};
use log::info;
use crate::config::config_loader::load_config;
use crate::runner::game_process_wrapper::run_game_process;
use crate::gui::show_gui;
use crate::install::install::check_install;

slint::include_modules!();

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new("info,tracing=warn"))
        .init();

    load_config();

    let is_in_steam_env = env::var("SteamEnv").unwrap_or("0".to_string()) == "1";

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
