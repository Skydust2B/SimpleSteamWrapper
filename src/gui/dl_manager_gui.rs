use std::rc::Rc;
use std::sync::{Arc, Mutex};
use log::info;
use slint::{ComponentHandle, ModelRc, SharedString, VecModel, Weak};
use tokio::fs;
use crate::compatibility_tools::compat_tools_list::{CompatToolsList};
use crate::dl_manager::dl_manager_installer::{download_and_extract_asset};
use crate::dl_manager::github_api::{fetch_github_releases};
use crate::{DlManagerGUI, MainGUI};
use crate::compatibility_tools::remote_compat_tools_provider::{RemoteCompatToolsProvider, REMOTE_COMPAT_TOOL_PROVIDERS};
use crate::dl_manager::downloadable_asset::DownloadableAsset;
use crate::compatibility_tools::updatable_compat_tool::UpdatableCompatTool;
use crate::slint_utils::ClonableModel;
use crate::steam::steam::get_steam_compat_tools_path;

fn release_model(can_be_updated: bool, display_name: &str, name: &str) -> (bool, bool, SharedString, SharedString) {
    let compat = CompatToolsList::get();
    let already_downloaded = compat.iter().find(|ct| ct.name.as_str() == name).is_some();

    (already_downloaded, can_be_updated, SharedString::from(display_name), SharedString::from(name))
}

fn fetch_releases_and_update_list_async(window: Weak<DlManagerGUI>, mutable_list: Arc<Mutex<Vec<DownloadableAsset>>>, provider: &RemoteCompatToolsProvider) {
    let compat_tool_path = provider.remote_path;
    let compat_tool_name = provider.name;
    tokio::spawn(async move {
        let rel = fetch_github_releases(compat_tool_path).await.unwrap();

        let assets_from_rels = rel.iter().fold(Vec::new(), |acc, r| {
            [acc, r.get_unique_assets()].concat()
        });

        let updatable_variant = {
            let updatable_variant_tool = UpdatableCompatTool::from_tool_name(compat_tool_name).await;
            let most_recent_rel = rel.first().unwrap().get_unique_assets();
            let most_recent_asset = most_recent_rel.first().unwrap();

            let mut converted = DownloadableAsset::from(most_recent_asset);
            converted.custom_folder = Some(updatable_variant_tool.path);
            converted.display_name = updatable_variant_tool.display_name;
            converted
        };
        let mut as_downloadable_assets = assets_from_rels.iter().map(|f| DownloadableAsset::from(f)).collect::<Vec<DownloadableAsset>>();
        as_downloadable_assets.insert(0, updatable_variant);

        let mut mutable_list = mutable_list.lock().unwrap();
        *mutable_list = as_downloadable_assets.clone();
        let _ = window.upgrade_in_event_loop(|w| w.invoke_update_ui_releases());
    });
}


pub fn show_gui(main_gui: Weak<MainGUI>) {
    let window = DlManagerGUI::new().unwrap();

    let providers_model = ClonableModel::new(REMOTE_COMPAT_TOOL_PROVIDERS.into());

    window.set_provider_names(providers_model.to_model_rc(|e| e.name.to_string()));

    let assets_release_list: Arc<Mutex<Vec<DownloadableAsset>>> = Arc::new(Mutex::new(Vec::new()));

    window.on_update_ui_releases({
        let weak_window = window.as_weak();
        let assets_release_list = assets_release_list.clone();
        let main_weak_window = main_gui.clone();
        move || {
            let assets_release_list = assets_release_list.clone();
            let main_weak_window = main_weak_window.clone();
            let _ = weak_window.upgrade_in_event_loop(move |window| {
                let mutable_list = assets_release_list.lock().unwrap();
                let model_base = mutable_list.iter().map(|v| {
                    release_model(v.custom_folder.is_some(), &v.display_name.clone(), &v.asset_name)
                }).collect::<VecModel<(bool, bool, SharedString, SharedString)>>();

                let model = Rc::new(VecModel::from(model_base));

                if let Some(main_window) = main_weak_window.upgrade() {
                    main_window.invoke_force_reload();
                }

                window.set_releases(model.into());
            });
        }
    });

    window.on_install_compat_tool({
        let weak_window = window.as_weak();
        let cloned_list = assets_release_list.clone();
        move |idx| {
            let env_window = weak_window.upgrade().unwrap();
            let cloned_list = cloned_list.lock().unwrap();

            let dct = cloned_list
                .iter()
                .nth(idx as usize)
                .cloned();

            if let Some(dc) = dct {
                // Recheck if already there
                let weak_window = env_window.as_weak();
                tokio::spawn(async move {
                    let _ = weak_window.upgrade_in_event_loop({
                        move |window| {
                            window.set_download_state((true, 0, idx));
                        }
                    });

                    let progress_db = Arc::new({
                        let weak_window = weak_window.clone();
                        move |downloaded, total_size| {
                            if let Some(window) = weak_window.upgrade() {
                                let percent = ((downloaded as f64 / total_size as f64) * 100.0).round() as i32;
                                window.set_download_state((true, percent, idx));
                            }
                        }
                    });

                    let _ = download_and_extract_asset(&dc, progress_db).await.unwrap();

                    // Update installed tools
                    CompatToolsList::refresh();
                    let _ = weak_window.upgrade_in_event_loop(|window| {
                        window.invoke_update_ui_releases();
                        window.set_download_state((false, 0, 0));
                    });
                });
            }
        }
    });

    window.on_fetch_release({
        let assets_release_list = assets_release_list.clone();
        let weak_window = window.as_weak();
        move |provider_idx, variant_idx| {
            let provider = providers_model.get_from_idx(provider_idx);
            fetch_releases_and_update_list_async(weak_window.clone(), assets_release_list.clone(), &provider)
        }
    });

    window.on_delete_compat_tool({
        let weak_window = window.as_weak();
        let cloned_list = assets_release_list.clone();
        move |idx| {
            let weak_window = weak_window.clone();
            let cloned_list = cloned_list.lock().unwrap();
            let found_item = cloned_list
                .iter()
                .nth(idx as usize)
                .cloned();

            if let Some(found_item) = found_item {
                tokio::spawn(async move {
                    let path = found_item.custom_folder.unwrap_or(get_steam_compat_tools_path().join(found_item.asset_name));
                    // TOdo: find in compat_tools_list to get path and remove properly
                    info!("Deleting compat_tool at {}", path.display());
                    let _ = fs::remove_dir_all(path).await;
                    CompatToolsList::refresh();

                    let _ = weak_window.upgrade_in_event_loop(|window| {
                        window.invoke_update_ui_releases();
                    });
                });
            }
        }
    });

    fetch_releases_and_update_list_async(window.as_weak(), assets_release_list.clone(), &REMOTE_COMPAT_TOOL_PROVIDERS[0]);

    let _ = window.show();
}
