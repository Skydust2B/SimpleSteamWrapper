use std::path::PathBuf;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use log::{error, info};
use slint::{ComponentHandle, Model, VecModel, Weak};
use tokio::fs;
use crate::compatibility_tools::compat_tools_list::{CompatToolsList};
use crate::dl_manager::dl_manager_installer::{download_and_extract_asset};
use crate::{DlAssetRow, DlManagerGUI, DownloadState, MainGUI};
use crate::compatibility_tools::remote_compat_tools_provider::{RemoteCompatToolsProvider, REMOTE_COMPAT_TOOL_PROVIDERS};
use crate::dl_manager::downloadable_asset::DownloadableAsset;
use crate::compatibility_tools::updatable_compat_tool::UpdatableCompatTool;
use crate::gui::globals::init_hard_refresh::{WindowForceRefresh};
use crate::utils::slint_utils::{ClonableModel};

async fn build_dl_assets_rows(provider: RemoteCompatToolsProvider, assets_release_list: Arc<Mutex<Vec<DownloadableAsset>>>) -> Vec<DlAssetRow> {
    let updatable_compat_tool = UpdatableCompatTool::from_tool_name(provider.name).await;
    let mut virtual_rows = Vec::from([
        DlAssetRow {
            id: 0,
            already_downloaded: updatable_compat_tool.local_version.is_some(),
            can_be_updated: true,
            display_name: updatable_compat_tool.display_name.clone().into(),
            name: updatable_compat_tool.name.clone().into()
        }
    ]);

    let compat = CompatToolsList::get();
    let mutable_list = assets_release_list.lock().unwrap();
    let mut idx = 0;
    let assets_row = mutable_list.iter().map(|v| {
        idx = idx + 1;
        DlAssetRow {
            id: idx,
            already_downloaded: compat.iter().find(|ct| ct.name.as_str() == &v.asset_name).is_some(),
            can_be_updated: false,
            display_name: v.display_name.clone().into(),
            name: v.asset_name.clone().into()
        }
    }).collect::<Vec<DlAssetRow>>();

    virtual_rows.extend(assets_row);

    virtual_rows
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
        let mut mutable_list = mutable_list.lock().unwrap();
        *mutable_list = as_downloadable_assets.clone();

        let _ = window.upgrade_in_event_loop(|w| w.invoke_update_ui_releases());
    });
}

async fn install_compat_tool(
    window: Weak<DlManagerGUI>,
    row: DlAssetRow,
    provider: &RemoteCompatToolsProvider,
    dl_assets_list: Arc<Mutex<Vec<DownloadableAsset>>>
) {
    let asset_to_download = if row.can_be_updated {
        let last_asset = {
             dl_assets_list.lock().unwrap()
                 .get(0).cloned()
        };
        if last_asset.is_none() {
            return;
        }
        let mut last_asset = last_asset.unwrap();

        let updatable_compat_tool = UpdatableCompatTool::from_tool_name(provider.name).await;
        last_asset.custom_folder = Some(updatable_compat_tool.path);
        Some(last_asset)
    } else {
        let asset_list = dl_assets_list.lock().unwrap();
        let asset = asset_list.iter().find(|asset| row.name == asset.asset_name).cloned();
        asset
    };

    if asset_to_download.is_none() {
        error!("No asset to download found.");
        return;
    }
    let asset_to_download = asset_to_download.unwrap();

    // Recheck if already there
    let weak_window = window.clone();
    let _ = weak_window.upgrade_in_event_loop({
        move |window| {
            window.set_download_state(DownloadState{
                is_downloading: true,
                percent: 0,
                row_id: row.id
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
                    row_id: row.id
                });
            }
        }
    });

    let _ = download_and_extract_asset(&asset_to_download, progress_db).await;

    // Update installed tools
    CompatToolsList::refresh();
    let _ = weak_window.upgrade_in_event_loop(|window| {
        window.invoke_update_ui_releases();
        window.set_download_state(DownloadState{
            is_downloading: false,
            percent: 0,
            row_id: 0
        });
    });
}

async fn delete_compat_tool(row: DlAssetRow) {
    let list = CompatToolsList::get();

    let compat_tool = list.iter().find(|compat| {
        compat.name == row.name.to_string()
    }).expect(&format!("Unable to find asset to remove for {:?}", &row));

    let path = PathBuf::from(compat_tool.dir_path.clone());

    info!("Deleting compat_tool at {}", path.display());

    let _ = fs::remove_dir_all(path).await;
    CompatToolsList::refresh();
}

pub fn show_gui(main_gui: Weak<MainGUI>) {
    let window = DlManagerGUI::new().unwrap();

    let providers_model = ClonableModel::new(REMOTE_COMPAT_TOOL_PROVIDERS.into());
    let variant_model: ClonableModel<String> = ClonableModel::new(providers_model
        .get_from_idx(0).unwrap()
        .variants.to_vec().iter()
        .map(|e| e.name.to_string()).collect());

    window.set_provider_names(providers_model.to_model_rc(|e| e.name.to_string()));
    window.set_variants(variant_model.to_model_rc(|variants| variants.clone()));

    let assets_release_list: Arc<Mutex<Vec<DownloadableAsset>>> = Arc::new(Mutex::new(Vec::new()));

    window.on_update_ui_releases({
        let weak_window = window.as_weak();
        let assets_release_list = assets_release_list.clone();
        let providers_model = providers_model.clone();

        move || {
            let assets_release_list = assets_release_list.clone();
            let current_provider: Option<RemoteCompatToolsProvider> = weak_window.upgrade().and_then(|w|
                providers_model.get_from_idx(w.get_current_provider_idx()));

            let weak_window = weak_window.clone();

            tokio::spawn(async move {
                if current_provider.is_none() {
                    return;
                }
                let rows = build_dl_assets_rows(current_provider.unwrap(), assets_release_list).await;
                let _ = weak_window.upgrade_in_event_loop(move |window| {
                    window.set_releases(Rc::<VecModel<DlAssetRow>>::new(rows.into()).into());
                });
            });
        }
    });

    window.on_install_compat_tool({
        let weak_window = window.as_weak();
        let cloned_list = assets_release_list.clone();
        let providers_model = providers_model.clone();
        let main_weak_window = main_gui.clone();

        move |idx| {
            let env_window = weak_window.upgrade();
            if env_window.is_none() {
                return;
            }
            let env_window = env_window.unwrap();
            let current_provider: Option<RemoteCompatToolsProvider> = providers_model.get_from_idx(env_window.get_current_provider_idx());
            let current_row = env_window.get_releases().iter().nth(idx as usize);

            if current_provider.is_none() || current_row.is_none() {
                return;
            }

            let main_weak_window = main_weak_window.clone();
            let cloned_list = cloned_list.clone();
            let weak_window = env_window.as_weak();

            tokio::spawn(async move {
                install_compat_tool(weak_window, current_row.unwrap(), &current_provider.unwrap(), cloned_list).await;

                // Reload the main GUI to update usable compat tools
                let _ = main_weak_window.upgrade_in_event_loop(|w| w.force_refresh());
            });
        }
    });

    window.on_fetch_releases({
        let assets_release_list = assets_release_list.clone();
        let weak_window = window.as_weak();
        let variant_model = variant_model.clone();
        let providers_model = providers_model.clone();

        move |provider_idx, variant_idx, update_variants, force_refresh| {
            let provider = providers_model.get_from_idx(provider_idx);
            if provider.is_none() {
                return;
            }
            let provider = provider.unwrap();

            if update_variants {
                let new_variants = provider.variants.to_vec().iter().map(|e| e.name.to_string()).collect();
                variant_model.set_model(new_variants);

                if let Some(window) = weak_window.upgrade() {
                    window.set_variants(variant_model.to_model_rc(|variants| variants.clone()));
                }
            }
            let selected_variant = variant_model.get_from_idx(variant_idx);
            if selected_variant.is_none() {
                return;
            }
            let selected_variant = selected_variant.unwrap();

            run_fetch_releases_update_list_task(weak_window.clone(), assets_release_list.clone(), &provider, &selected_variant, force_refresh)
        }
    });

    window.on_delete_compat_tool({
        let weak_window = window.as_weak();

        let main_weak_window = main_gui.clone();

        move |idx| {
            let weak_window = weak_window.clone();
            let main_weak_window = main_weak_window.clone();

            let current_row = weak_window.upgrade()
                .and_then(|w| w.get_releases().iter().nth(idx as usize));
            if current_row.is_none() {
                return;
            }

            if let Some(found_item) = current_row {
                tokio::spawn(async move {
                    delete_compat_tool(found_item).await;

                    let _ = weak_window.upgrade_in_event_loop(|window| {
                        window.invoke_update_ui_releases();
                    });
                    let _ = main_weak_window.upgrade_in_event_loop(|w| w.force_refresh());
                });
            }
        }
    });

    run_fetch_releases_update_list_task(window.as_weak(), assets_release_list.clone(), &REMOTE_COMPAT_TOOL_PROVIDERS[0], &variant_model.get_from_idx(0).unwrap(), false);

    let _ = window.show();
}
