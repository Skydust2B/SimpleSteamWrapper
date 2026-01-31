use std::rc::Rc;
use std::sync::{Arc, Mutex};
use log::info;
use slint::{ComponentHandle, ModelRc, SharedString, VecModel, Weak};
use tokio::fs;
use crate::compatibility_tools::steam::get_steam_compat_tools_path;
use crate::compatibility_tools::steam_compat_tools_list::SteamCompatToolsList;
use crate::dl_manager::dl_manager::{download_and_extract_asset};
use crate::dl_manager::github_api::{fetch_github_releases, SimplifiedGithubAsset};
use crate::dl_manager::remote_compat_tools::{DownloadableCompatTool, DOWNLOADABLE_COMPAT_TOOLS};
use crate::{DlManagerGUI};
use crate::io_utils::strip_all_extensions;

fn release_model(auto_update: bool, name: &str) -> (bool, bool, SharedString) {
    let compat = SteamCompatToolsList::get_list();
    let already_downloaded = compat.iter().find(|ct| ct.name.as_str() == name).is_some();

    (already_downloaded, auto_update, SharedString::from(name))
}

pub fn show_gui() {
    let window = DlManagerGUI::new().unwrap();

    let model: ModelRc<SharedString> = Rc::new(
        VecModel::from(DOWNLOADABLE_COMPAT_TOOLS.iter().map(|dc| SharedString::from(dc.name)).collect::<Vec<SharedString>>())
    ).into();

    window.set_dl_compat_tools(model);

    let assets_release_list: Arc<Mutex<Vec<SimplifiedGithubAsset>>> = Arc::new(Mutex::new(Vec::new()));

    window.on_update_ui_releases({
        let weak_window = window.as_weak();
        let assets_release_list = assets_release_list.clone();
        move || {
            let assets_release_list = assets_release_list.clone();
            let _ = weak_window.upgrade_in_event_loop(move |window| {
                let mutable_list = assets_release_list.lock().unwrap();
                let model_base = mutable_list.iter().map(|v| {
                    release_model(false, strip_all_extensions(&v.name.clone()))
                }).collect::<VecModel<(bool, bool, SharedString)>>();

                let model = Rc::new(VecModel::from(model_base));

                window.set_releases(model.into());
            });
        }
    });

    let fetch_releases_async = |window: Weak<DlManagerGUI>, mutable_list: Arc<Mutex<Vec<SimplifiedGithubAsset>>>, compat_tool: &DownloadableCompatTool| {
        let compat_tool_path = compat_tool.remote_path;
        tokio::spawn(async move {
            let rel = fetch_github_releases(compat_tool_path).await.unwrap();

            let updatable_variant = SimplifiedGithubAsset {
                id: 0,
                name: "".to_string(),
                browser_download_url: "".to_string(),
                content_type: "".to_string(),
                created_at: "".to_string(),
            };
            let assets_from_rels = rel.iter().fold(Vec::new(), |acc, r| {
                [acc, r.get_unique_assets()].concat()
            });

            let mut mutable_list = mutable_list.lock().unwrap();
            *mutable_list = assets_from_rels.clone();
            let _ = window.upgrade_in_event_loop(|w| w.invoke_update_ui_releases());
        });
    };

    window.on_install_compat_tool({
        let weak_window = window.as_weak();
        let cloned_list = assets_release_list.clone();
        move |v| {
            let env_window = weak_window.upgrade().unwrap();
            let cloned_list = cloned_list.lock().unwrap();
            let dct = cloned_list
                .iter()
                .find(|c| c.name_without_ext() == v.as_str())
                .cloned();

            if let Some(dc) = dct {
                // Recheck if already there
                let weak_window = env_window.as_weak();
                tokio::spawn(async move {
                    let _ = weak_window.upgrade_in_event_loop({
                        let name = SharedString::from(dc.name_without_ext());
                        move |window| {
                            window.set_download_state((name.clone(), true, 0));
                        }
                    });

                    let name = SharedString::from(dc.name_without_ext());
                    let progress_db = Arc::new({
                        let weak_window = weak_window.clone();
                        move |downloaded, total_size| {
                            if let Some(window) = weak_window.upgrade() {
                                let percent = ((downloaded as f64 / total_size as f64) * 100.0).round() as i32;
                                window.set_download_state((name.clone(), true, percent));
                            }
                        }
                    });

                    let _ = download_and_extract_asset(&dc, progress_db).await.unwrap();

                    // Update installed tools
                    SteamCompatToolsList::refresh_list();
                    let _ = weak_window.upgrade_in_event_loop(|window| {
                        window.invoke_update_ui_releases();
                        window.set_download_state((SharedString::new(), false, 0));
                    });
                });
            }
        }
    });

    window.on_fetch_release({
        let assets_release_list = assets_release_list.clone();
        let weak_window = window.as_weak();
        move |v| {
            let dct = DOWNLOADABLE_COMPAT_TOOLS.iter().find(|c| c.name == v.as_str());
            if let Some(dc) = dct {
                fetch_releases_async(weak_window.clone(), assets_release_list.clone(), dc)
            }
        }
    });

    window.on_delete_compat_tool({
        let weak_window = window.as_weak();
        move |v| {
            let weak_window = weak_window.clone();
            if !v.is_empty() {
                tokio::spawn(async move {
                    let path = get_steam_compat_tools_path().join(v.as_str());
                    info!("Deleting compat_tool at {}", path.display());
                    let _ = fs::remove_dir_all(path).await;
                    SteamCompatToolsList::refresh_list();

                    let _ = weak_window.upgrade_in_event_loop(|window| {
                        window.invoke_update_ui_releases();
                    });
                });
            }
        }
    });

    fetch_releases_async(window.as_weak(), assets_release_list.clone(), &DOWNLOADABLE_COMPAT_TOOLS[0]);

    let _ = window.show();
}
