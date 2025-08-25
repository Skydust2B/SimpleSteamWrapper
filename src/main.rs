extern crate core;

mod gui;
mod tweaks;
mod gpu;
mod config;
mod runner;

use tracing_subscriber::EnvFilter;
use device_query::{DeviceQuery, DeviceState, Keycode};
use crate::config::config_loader::load_config;
use crate::runner::game_process_wrapper::run_game_process;
use crate::gui::show_gui;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new("info,tracing=warn"))
        .init();

    load_config();

    // 1. Spawn thread to check for key hold and show GUI
    let device_state = DeviceState::new();
    let keys: Vec<Keycode> = device_state.get_keys();
    if keys.contains(&Keycode::LShift) { // hold Shift to show GUI
        show_gui();
    }
    run_game_process();
}
