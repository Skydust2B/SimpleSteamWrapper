use std::process::Command;
use tweaks_macro::tweak;
use crate::tweak_collector::PreparedCommand;
use crate::utils::command_utils::UpdateEnvVar;

#[tweak(name = "no_dx12_dxr")]
pub fn run(process: &mut Command, _: &mut PreparedCommand) {
    process.add_parameter_to_var(",", "VKD3D_CONFIG", "nodxr");
}
