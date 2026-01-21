use std::rc::Rc;
use std::sync::{Arc, Mutex};
use slint::{ComponentHandle, ModelRc, SharedString, VecModel, Weak};
use crate::dl_manager::dl_manager::download_and_extract_release;
use crate::dl_manager::github_api::{fetch_github_releases, SimplifiedGithubRelease};
use crate::dl_manager::remote_compat_tools::{DownloadableCompatTool, DOWNLOADABLE_COMPAT_TOOLS};
use crate::DlManagerGUI;

pub fn show_gui() {
    let window = DlManagerGUI::new().unwrap();

    let model: ModelRc<SharedString> = Rc::new(
        VecModel::from(DOWNLOADABLE_COMPAT_TOOLS.iter().map(|dc| SharedString::from(dc.name)).collect::<Vec<SharedString>>())
    ).into();

    window.set_dl_compat_tools(model);

    let release_list: Arc<Mutex<Vec<SimplifiedGithubRelease>>> = Arc::new(Mutex::new(Vec::new()));

    window.set_releases(Default::default());

    let invoke_window_from_event_loop = |
        weak: Weak<DlManagerGUI>,
        cb: fn(DlManagerGUI)
    | {
        let _ = slint::invoke_from_event_loop({
        move || {
          if let Some(window) = weak.upgrade() {
              cb(window);
          }
        }});
    };

    window.on_install_compat_tool({
        let weak_window = window.as_weak();
        let cloned_list = release_list.clone();
        move |v| {
            let env_window = weak_window.upgrade().unwrap();
            let cloned_list = cloned_list.lock().unwrap();
            let dct = cloned_list
                .iter()
                .find(|c| c.name == v.as_str())
                .cloned();

            if let Some(dc) = dct {
                // Recheck if already there
                let weak_window = env_window.as_weak();
                tokio::spawn(async move {
                    invoke_window_from_event_loop(weak_window.clone(), |window| {
                        window.set_download_state((true, 0));
                    });

                    let progress_db = Arc::new({
                        let weak_window = weak_window.clone();
                        move |downloaded, total_size| {
                            if let Some(window) = weak_window.upgrade() {
                                let percent = ((downloaded as f64 / total_size as f64) * 100.0).round() as i32;
                                window.set_download_state((true, percent));
                            }
                        }
                    });

                    let _ = download_and_extract_release(&dc, progress_db).await.unwrap();

                    // Update installed tools

                    invoke_window_from_event_loop(weak_window.clone(), |window| {
                        window.set_download_state((false, 0));
                    });
                });
            }
        }
    });

    let fetch_release_async = |mutable_list: Arc<Mutex<Vec<SimplifiedGithubRelease>>>, compat_tool: &DownloadableCompatTool| {
            let compat_tool_path = compat_tool.remote_path.clone();
            tokio::spawn(async move {
                let rel = fetch_github_releases(compat_tool_path).await.unwrap();
                let mut mutable_list = mutable_list.lock().unwrap();
                *mutable_list = rel;
            });
        };

    window.on_fetch_release({
        let release_list = release_list.clone();
        move |v| {
            let dct = DOWNLOADABLE_COMPAT_TOOLS.iter().find(|c| c.name == v.as_str());
            if let Some(dc) = dct {
                fetch_release_async(release_list, dc)
            }
        }
    });

    let _ = window.show();
}
