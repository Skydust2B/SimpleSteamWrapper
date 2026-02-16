use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;
use once_cell::sync::Lazy;
use crate::steam::steam::{get_steam_path, read_vdf};

#[derive(Clone)]
pub struct InstalledSteamApp {
    #[allow (dead_code)]
    pub id: String,
    pub name: String,
    pub path: PathBuf
}

static INSTALLED_STEAM_APPS: Lazy<Mutex<HashMap<String, InstalledSteamApp>>> = Lazy::new(|| {
    let apps = get_installed_steam_apps_inner();
    Mutex::new(apps)
});

fn get_installed_steam_apps_inner() -> HashMap<String, InstalledSteamApp> {
    let library_vdf_path = get_steam_path().unwrap().join("steamapps/libraryfolders.vdf");
    let library_vdf = read_vdf(&library_vdf_path).expect(format!("Could not read libraries from VDF {}", library_vdf_path.display()).as_str());
    let libraries = library_vdf["libraryfolders"].as_table().unwrap().values();

    let mut installed_steam_apps: HashMap<String, InstalledSteamApp> = HashMap::new();

    for library in libraries {
        let library_path = PathBuf::from(library.get("path").unwrap().as_str().unwrap());
        let apps = library.get("apps").unwrap().as_table().unwrap().keys();

        for app in apps {
            let app_manifest_path = library_path.join(format!("steamapps/appmanifest_{}.acf", app));
            if let Ok(app_manifest) = read_vdf(&app_manifest_path) {
                let manifest_root = app_manifest.get("AppState").unwrap().as_table().unwrap();
                installed_steam_apps.insert(app.to_string(), InstalledSteamApp {
                    id: app.to_string(),
                    name: manifest_root.get("name").unwrap().as_str().unwrap().to_string(),
                    path: library_path
                        .join("steamapps")
                        .join("common")
                        .join(manifest_root.get("installdir").unwrap().as_str().unwrap())
                });
            }
        }
    }

    installed_steam_apps
}

pub fn get_installed_steam_apps() -> HashMap<String, InstalledSteamApp> {
    INSTALLED_STEAM_APPS.lock().unwrap().clone()
}
