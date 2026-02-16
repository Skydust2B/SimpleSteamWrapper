use std::collections::HashMap;
use std::iter::Map;
use crate::dl_manager::downloadable_asset::DownloadableAsset;
use crate::dl_manager::github_api::fetch_github_releases;

#[derive(Debug)]
#[derive(Clone)]
pub struct RemoteCompatToolsProvider {
    pub name: &'static str,
    pub remote_path: &'static str,
    pub variants: &'static [&'static str]
}

impl RemoteCompatToolsProvider {

    pub async fn fetch_assets_by_variant_name(
        &self
    ) -> anyhow::Result<HashMap<String, Vec<DownloadableAsset>>> {
        let assets = self.fetch_unique_assets().await?; // handle Result

        let mut map: HashMap<String, Vec<DownloadableAsset>> = HashMap::new();

        for asset in &assets {
            for variant in self.variants {
                if asset.asset_name.contains(variant) {
                    map.entry(variant.to_string())
                        .or_insert_with(Vec::new)
                        .push(asset.clone());
                }
            }
        }

        Ok(map)
    }

    pub async fn fetch_unique_assets(&self) -> anyhow::Result<Vec<DownloadableAsset>> {
        let rel = fetch_github_releases(self.remote_path).await?;

        let assets = rel.iter()
            .fold(Vec::new(), |acc, r| { [acc, r.get_unique_assets()].concat() })
            .iter()
            .map(|f| DownloadableAsset::from(f))
            .collect::<Vec<DownloadableAsset>>();

        Ok(assets)
    }
}

pub const REMOTE_COMPAT_TOOL_PROVIDERS: &[RemoteCompatToolsProvider] = &[
    RemoteCompatToolsProvider {
        name: "proton-cachyos",
        remote_path: "CachyOS/proton-cachyos",
        variants: &["slr-x86_64", "slr-x86_64_v2", "slr-x86_64_v3", "slr-x86_64_v4"]
    },
    RemoteCompatToolsProvider{
        name: "GE-Proton",
        remote_path: "GloriousEggroll/proton-ge-custom",
        variants: &[""]
    }
];
