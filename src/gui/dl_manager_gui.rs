use std::rc::Rc;
use std::sync::{Arc, Mutex};
use log::info;
use slint::{ComponentHandle, ModelRc, SharedString, VecModel, Weak};
use crate::compatibility_tools::steam::list_steam_compat_tools;
use crate::dl_manager::dl_manager::download_and_extract_release;
use crate::dl_manager::github_api::{fetch_github_releases, SimplifiedGithubRelease};
use crate::dl_manager::remote_compat_tools::{DownloadableCompatTool, DOWNLOADABLE_COMPAT_TOOLS};
use crate::DlManagerGUI;
use crate::gui::gui_utils::InvokeFromEventLoop;

fn release_model(auto_update: bool, parent: &str, name: &str) -> (bool, bool, SharedString, SharedString) {
    let compat = list_steam_compat_tools();
    let already_downloaded = compat.iter().find(|ct| ct.name.as_str() == name).is_some();
    info!("TODO: Change this check for a cached one");

    (already_downloaded, auto_update, SharedString::from(name), SharedString::from(parent))
}

pub fn show_gui() {
    let window = DlManagerGUI::new().unwrap();

    let model: ModelRc<SharedString> = Rc::new(
        VecModel::from(DOWNLOADABLE_COMPAT_TOOLS.iter().map(|dc| SharedString::from(dc.name)).collect::<Vec<SharedString>>())
    ).into();

    window.set_dl_compat_tools(model);

    let release_list: Arc<Mutex<Vec<SimplifiedGithubRelease>>> = Arc::new(Mutex::new(Vec::new()));

    window.on_install_compat_tool({
        let weak_window = window.as_weak();
        let cloned_list = release_list.clone();
        move |v| {
            let env_window = weak_window.upgrade().unwrap();
            let cloned_list = cloned_list.lock().unwrap();
            let dct = cloned_list
                .iter()
                .find(|c| c.tag_name == v.as_str())
                .cloned();

            if let Some(dc) = dct {
                // Recheck if already there
                let weak_window = env_window.as_weak();
                tokio::spawn(async move {
                    weak_window.invoke({
                        let name = SharedString::from(dc.tag_name.clone());
                        move |window| {
                            window.set_download_state((name.clone(), true, 0));
                        }
                    });

                    let name = SharedString::from(dc.tag_name.clone());
                    let progress_db = Arc::new({
                        let weak_window = weak_window.clone();
                        move |downloaded, total_size| {
                            if let Some(window) = weak_window.upgrade() {
                                let percent = ((downloaded as f64 / total_size as f64) * 100.0).round() as i32;
                                window.set_download_state((name.clone(), true, percent));
                            }
                        }
                    });

                    let _ = download_and_extract_release(&dc, progress_db).await.unwrap();

                    // Update installed tools
                    weak_window.invoke(|window| {
                        window.set_download_state((SharedString::new(), false, 0));
                    });
                });
            }
        }
    });

    let fetch_releases_async = |window: Weak<DlManagerGUI>, mutable_list: Arc<Mutex<Vec<SimplifiedGithubRelease>>>, compat_tool: &DownloadableCompatTool| {
            let compat_tool_path = compat_tool.remote_path;
            let compat_tool_name = compat_tool.name;
            tokio::spawn(async move {
                let rel = fetch_github_releases(compat_tool_path).await.unwrap();
                let mut mutable_list = mutable_list.lock().unwrap();
                *mutable_list = rel.clone();


                window.invoke(move |window| {
                    let model_base = rel.iter().map(|v| {
                        release_model(false, &compat_tool_name, &v.tag_name.clone())
                    }).collect::<VecModel<(bool, bool, SharedString, SharedString)>>();

                    let model = Rc::new(VecModel::from(model_base));

                    window.set_releases(model.into());
                })
            });
        };

    window.on_fetch_release({
        let release_list = release_list.clone();
        let weak_window = window.as_weak();
        move |v| {
            let dct = DOWNLOADABLE_COMPAT_TOOLS.iter().find(|c| c.name == v.as_str());
            if let Some(dc) = dct {
                fetch_releases_async(weak_window.clone(), release_list.clone(), dc)
            }
        }
    });

    fetch_releases_async(window.as_weak(), release_list.clone(), &DOWNLOADABLE_COMPAT_TOOLS[0]);

    let _ = window.show();
}
