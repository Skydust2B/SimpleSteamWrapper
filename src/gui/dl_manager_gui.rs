use std::path::PathBuf;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use log::info;
use slint::{ComponentHandle, VecModel, Weak};
use tokio::fs;
use crate::compatibility_tools::compat_tools_list::{CompatToolsList};
use crate::dl_manager::dl_manager_installer::{download_and_extract_asset};
use crate::{DlManagerGUI, DownloadState, MainGUI, Release};
use crate::compatibility_tools::remote_compat_tools_provider::{RemoteCompatToolsProvider, REMOTE_COMPAT_TOOL_PROVIDERS};
use crate::dl_manager::downloadable_asset::DownloadableAsset;
use crate::compatibility_tools::updatable_compat_tool::UpdatableCompatTool;
use crate::gui::globals::init_hard_refresh::{WindowForceRefresh};
use crate::utils::slint_utils::{ClonableModel, WeakUtils};

fn release_model(can_be_updated: bool, display_name: &str, name: &str) -> Release {
    let compat = CompatToolsList::get();
    let already_downloaded = compat.iter().find(|ct| ct.name.as_str() == name).is_some();

    Release {
        already_downloaded,
        can_be_updated,
        display_name: display_name.into(),
        name: name.into()
    }
}

fn run_fetch_releases_update_list_task(
    window: Weak<DlManagerGUI>,
    mutable_list: Arc<Mutex<Vec<DownloadableAsset>>>,
    provider: &RemoteCompatToolsProvider,
    selected_variant: &String,
    force_refresh: bool
) {
    let cloned_provider = provider.clone();
    let cloned_variant = selected_variant.clone();

    tokio::spawn(async move {
        let mut rel = cloned_provider.fetch_assets_by_variant_name(force_refresh).await.unwrap();

        let as_downloadable_assets = rel.get_mut(&cloned_variant.to_string()).expect("Variant not found.");
        let updatable_variant = {
            let updatable_variant_tool = UpdatableCompatTool::from_tool_name(cloned_provider.name).await;
            let mut converted = as_downloadable_assets.first().unwrap().clone();
            converted.custom_folder = Some(updatable_variant_tool.path);
            converted.display_name = updatable_variant_tool.display_name;
            converted
        };
        as_downloadable_assets.insert(0, updatable_variant);

        let mut mutable_list = mutable_list.lock().unwrap();
        *mutable_list = as_downloadable_assets.clone();
        let _ = window.upgrade_in_event_loop(|w| w.invoke_update_ui_releases());
    });
}


pub fn show_gui(main_gui: Weak<MainGUI>) {
    let window = DlManagerGUI::new().unwrap();

    let providers_model = ClonableModel::new(REMOTE_COMPAT_TOOL_PROVIDERS.into());
    let variant_model: ClonableModel<String> = ClonableModel::new(providers_model
        .get_from_idx(0).
        variants.to_vec().iter()
        .map(|e| e.name.to_string()).collect());

    window.set_provider_names(providers_model.to_model_rc(|e| e.name.to_string()));
    window.set_variants(variant_model.to_model_rc(|variants| variants.clone()));

    let assets_release_list: Arc<Mutex<Vec<DownloadableAsset>>> = Arc::new(Mutex::new(Vec::new()));

    window.on_update_ui_releases({
        let weak_window = window.as_weak();
        let assets_release_list = assets_release_list.clone();
        let main_weak_window = main_gui.clone();
        move || {
            let assets_release_list = assets_release_list.clone();
            let main_weak_window = main_weak_window.clone();
            weak_window.upgrade_and_run(move |window| {
                let mutable_list = assets_release_list.lock().unwrap();
                let model_base = mutable_list.iter().map(|v| {
                    release_model(v.custom_folder.is_some(), &v.display_name.clone(), &v.asset_name)
                }).collect::<VecModel<Release>>();

                let model = Rc::new(model_base);

                main_weak_window.upgrade_and_run(|w| w.force_refresh());

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
                            window.set_download_state(DownloadState{
                                is_downloading: true,
                                percent: 0,
                                row_id: idx
                            });
                        }
                    });

                    let progress_db = Arc::new({
                        let weak_window = weak_window.clone();
                        move |downloaded, total_size| {
                            if let Some(window) = weak_window.upgrade() {
                                let percent = ((downloaded as f64 / total_size as f64) * 100.0).round() as i32;
                                window.set_download_state(DownloadState{
                                    is_downloading: true,
                                    percent,
                                    row_id: idx
                                });
                            }
                        }
                    });

                    let _ = download_and_extract_asset(&dc, progress_db).await.unwrap();

                    // Update installed tools
                    CompatToolsList::refresh();
                    let _ = weak_window.upgrade_in_event_loop(|window| {
                        window.invoke_fetch_release(window.get_current_provider_idx(), window.get_current_variant_idx(), false, true);
                        window.set_download_state(DownloadState{
                            is_downloading: false,
                            percent: 0,
                            row_id: 0
                        });
                    });
                });
            }
        }
    });

    window.on_fetch_release({
        let assets_release_list = assets_release_list.clone();
        let weak_window = window.as_weak();
        let variant_model = variant_model.clone();

        move |provider_idx, variant_idx, update_variants, force_refresh| {
            let provider = providers_model.get_from_idx(provider_idx);

            if update_variants {
                let new_variants = provider.variants.to_vec().iter().map(|e| e.name.to_string()).collect();
                variant_model.set_model(new_variants);

                if let Some(window) = weak_window.upgrade() {
                    window.set_variants(variant_model.to_model_rc(|variants| variants.clone()));
                }
            }

            let selected_variant = variant_model.get_from_idx(variant_idx);
            run_fetch_releases_update_list_task(weak_window.clone(), assets_release_list.clone(), &provider, &selected_variant, force_refresh)
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
                    let list = CompatToolsList::get();

                    let compat_tool = list.iter().find(|compat| {
                        if let Some(custom_folder) = &found_item.custom_folder {
                            return compat.name == custom_folder.file_name().unwrap().to_str().unwrap()
                        }
                        return compat.name == found_item.asset_name;
                    }).expect(&format!("Unable to find asset to remove for {:?}", &found_item));

                    let path = PathBuf::from(compat_tool.dir_path.clone());
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

    run_fetch_releases_update_list_task(window.as_weak(), assets_release_list.clone(), &REMOTE_COMPAT_TOOL_PROVIDERS[0], &variant_model.get_from_idx(0), false);

    let _ = window.show();
}
