use log::info;
use slint::ComponentHandle;
use crate::dl_manager::github_api::fetch_github_releases;
use crate::dl_manager::remote_compat_tools::DOWNLOADABLE_COMPAT_TOOLS;
use crate::DlManagerGUI;

pub fn show_gui() {
    let window = DlManagerGUI::new().unwrap();

    for compat_tool in DOWNLOADABLE_COMPAT_TOOLS {
        tokio::spawn(async move {
            let rel = fetch_github_releases(compat_tool.remote_path).await.unwrap();
            info!("{} : Got github release: {:?}", compat_tool.name, rel);
        });
    }
    let _ = window.show();
}
