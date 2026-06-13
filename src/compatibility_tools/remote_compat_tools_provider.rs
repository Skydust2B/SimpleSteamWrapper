use std::collections::HashMap;
use std::sync::Mutex;
use once_cell::sync::Lazy;
use regex::Regex;
use crate::dl_manager::downloadable_asset::DownloadableAsset;
use crate::dl_manager::github_api::fetch_github_releases;

#[derive(Debug,Clone)]
pub struct RemoteCompatToolVariant {
    pub name: &'static str,
    pub regex: &'static str
}

#[derive(Debug,Clone)]
pub struct RemoteCompatToolsProvider {
    pub name: &'static str,
    pub remote_path: &'static str,
    pub variants: &'static [&'static RemoteCompatToolVariant]
}

static ASSETS_CACHE: Lazy<Mutex<HashMap<String, Vec<DownloadableAsset>>>> = Lazy::new(|| Mutex::new(HashMap::new()));

impl RemoteCompatToolsProvider {

    pub async fn fetch_assets_by_variant_name(
        &self,
        force_refresh: bool
    ) -> anyhow::Result<HashMap<String, Vec<DownloadableAsset>>> {
        let assets = self.fetch_assets(force_refresh).await?; // handle Result

        let mut map: HashMap<String, Vec<DownloadableAsset>> = HashMap::new();

        for asset in &assets {
            for variant in self.variants.iter().rev() {
                let variant_rgx = Regex::new(variant.regex)?;
                if !variant_rgx.is_match(&asset.asset_name){
                    continue;
                }

                map.entry(variant.name.to_string())
                    .or_insert_with(Vec::new)
                    .push(asset.clone());
                break;
            }
        }

        Ok(map)
    }

    pub async fn fetch_assets(&self, force_refresh: bool) -> anyhow::Result<Vec<DownloadableAsset>> {
        if !force_refresh {
            let assets_cache = Self::get_cache(self.remote_path.to_string());

            if let Some (assets_cache) = assets_cache {
                return Ok(assets_cache)
            }
        }

        let rel = fetch_github_releases(self.remote_path).await?;
        let assets = rel.iter()
            .fold(Vec::new(), |acc, r| { [acc, r.get_unique_assets()].concat() })
            .iter()
            .map(|f| DownloadableAsset::from(f))
            .collect::<Vec<DownloadableAsset>>();

        Self::update_cache(self.remote_path.to_string(), assets.clone());

        Ok(assets)
    }

    fn update_cache(path: String, values: Vec<DownloadableAsset>) {
        let mut assets_cache = ASSETS_CACHE.lock().expect("Failed to lock asset_cache mutex");
        assets_cache.insert(path, values);
    }

    fn get_cache(path: String) -> Option<Vec<DownloadableAsset>> {
        let assets_cache = ASSETS_CACHE.lock().expect("Failed to lock asset_cache mutex");
        if  !assets_cache.contains_key(path.as_str()) {
            return None;
        }
        Some(assets_cache.get(path.as_str()).unwrap().clone())
    }
}

pub const REMOTE_COMPAT_TOOL_PROVIDERS: &[RemoteCompatToolsProvider] = &[
    RemoteCompatToolsProvider {
        name: "proton-cachyos",
        remote_path: "CachyOS/proton-cachyos",
        variants: &[&RemoteCompatToolVariant{
            name: "slr x86_64",
            regex: "slr-x86_64"
        }, &RemoteCompatToolVariant{
            name: "slr x86_64_v2",
            regex: "slr-x86_64_v2"
        }, &RemoteCompatToolVariant{
            name: "slr x86_64_v3",
            regex: "slr-x86_64_v3"
        }]
    },
    RemoteCompatToolsProvider{
        name: "GE-Proton",
        remote_path: "GloriousEggroll/proton-ge-custom",
        variants: &[&RemoteCompatToolVariant{
            name: "Default",
            regex: ""
        }]
    }
];
