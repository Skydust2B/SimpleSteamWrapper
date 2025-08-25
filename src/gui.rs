use std::collections::HashMap;
use eframe::egui;
use eframe::glow::Context;
use egui::ComboBox;
use log::info;
use crate::config::config_loader::LOADED_CONFIG;
use crate::gpu::{get_formatted_gpu_id, get_gpu_from_config, list_all_gpus, GPU};
use crate::runner::compat_tools_wrapper::{get_compat_tool_from_config, list_steam_compat_tools, CompatTool};

struct MyApp {
    gpus: Vec<GPU>,
    selected_gpu_index: usize,
    compat_tools: Vec<CompatTool>,
    selected_compat_tool_index: usize
}

impl Default for MyApp {
    fn default() -> Self {
        let gpus = list_all_gpus();
        for t in &gpus {
            info!("{:?}", t);
            info!("0x{:04x}:0x{:04x}", t.vendor_id, t.device_id);
        }

        let compat_tools = list_steam_compat_tools();
        let selected_compat_tool_index = compat_tools.iter().position(|ct| get_compat_tool_from_config().name == ct.name).unwrap();
        let gpu = get_gpu_from_config();
        let selected_gpu_index = gpus.iter().position(|g| get_formatted_gpu_id(&gpu) == get_formatted_gpu_id(g)).unwrap();
        Self {
            gpus,
            selected_gpu_index,
            compat_tools,
            selected_compat_tool_index
        }
    }
}

impl MyApp {
    pub fn new(_: &eframe::CreationContext<'_>) -> Self {
        Default::default()
    }
}

fn get_safe_entry<'a>(map: &'a mut HashMap<String, bool>, key: &str) -> &'a mut bool {
    map.entry(key.to_string()).or_insert(false)
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("Choose runner (Use ProtonUpQt to add more)");
            ComboBox::from_label("Select Runner")
                .selected_text(&self.compat_tools[self.selected_compat_tool_index].name)
                .show_ui(ui, |ui| {
                    for (i, compat_tool) in self.compat_tools.iter().enumerate() {
                        ui.selectable_value(&mut self.selected_compat_tool_index, i, &compat_tool.name);
                    }
                });

            let mut conf = LOADED_CONFIG.get_config();

            let selected_compat_tool = &self.compat_tools[self.selected_compat_tool_index];
            conf.defaults.compat_tool = selected_compat_tool.name.to_string();

            let gpu_selection = get_safe_entry(&mut conf.defaults.enabled_tweaks, "select_gpu");
            ui.checkbox(
                gpu_selection,
                "Select Vulkan GPU (Using multiple env vars)"
            );
            ui.add_enabled_ui(*gpu_selection, |ui|
                           ComboBox::from_label("Select")
                               .selected_text(&self.gpus[self.selected_gpu_index].full_name)
                               .show_ui(ui, |ui| {
                                    for (i, gpu) in self.gpus.iter().enumerate() {
                                        ui.selectable_value(&mut self.selected_gpu_index, i, &gpu.full_name);
                                    }
                                }));

            let selected_gpu = &self.gpus[self.selected_gpu_index];
            conf.defaults.selected_gpu = get_formatted_gpu_id(selected_gpu);

            ui.checkbox(
                get_safe_entry(&mut conf.defaults.enabled_tweaks, "proton_nvapi"),
                "Use NVAPI extensions (PROTON_USE_NVAPI)"
            );

            ui.checkbox(
                get_safe_entry(&mut conf.defaults.enabled_tweaks, "gamemode"),
                "Feral's Gamemode"
            );

            ui.checkbox(
                get_safe_entry(&mut conf.defaults.enabled_tweaks, "mangohud"),
                "Use MangoHUD"
            );

            ui.checkbox(
                get_safe_entry(&mut conf.defaults.enabled_tweaks, "gamescope"),
                "Use Gamescope"
            );

            LOADED_CONFIG.set_config(conf);
        });
    }
    fn on_exit(&mut self, _gl: Option<&Context>) {
        LOADED_CONFIG.save();
    }
}

pub fn show_gui() {
    let options = eframe::NativeOptions::default();
    let _ = eframe::run_native(
        "Rust Boilerplate GUI",
        options,
        Box::new(|cc| Ok(Box::new(MyApp::new(cc)))),
    );
}
