use std::path::PathBuf;
use crate::dl_manager::github_api::SimplifiedGithubAsset;

#[derive(Debug, Clone)]
pub struct DownloadableAsset {
    pub display_name: String,
    pub asset_name: String,
    pub browser_download_url: String,
    pub content_type: String,
    pub custom_folder: Option<PathBuf>
}

impl From<&SimplifiedGithubAsset> for DownloadableAsset {
    fn from(asset: &SimplifiedGithubAsset) -> Self {
        Self {
            asset_name: asset.name_without_ext().to_string(),
            display_name: asset.name_without_ext().to_string(),
            custom_folder: None,
            browser_download_url: asset.browser_download_url.clone(),
            content_type: asset.content_type.clone()
        }
    }
}
