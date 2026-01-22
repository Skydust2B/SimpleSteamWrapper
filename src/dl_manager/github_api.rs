use serde::{Deserialize, Serialize};
use crate::io_utils::strip_all_extensions;

#[derive(Debug,Serialize,Deserialize,Clone)]
pub struct SimplifiedGithubAsset {
    pub id: usize,
    pub name: String,
    pub browser_download_url: String,
    pub(crate) content_type: String, // 	"application/x-xz"(.tar.xz) for cachyos and "application/zstd"/"application/gzip" (.tar.zst/.tar.gz)
    pub created_at: String
}

#[derive(Debug,Serialize,Deserialize,Clone)]
pub struct SimplifiedGithubRelease {
    pub id: usize,
    pub name: String,
    pub tag_name: String,
    pub published_at: String,
    pub(crate) assets: Vec<SimplifiedGithubAsset>
}

impl SimplifiedGithubAsset {
    pub fn name_without_ext(&self) -> &str {
        strip_all_extensions(&self.name)
    }
}

impl SimplifiedGithubRelease {
    pub fn get_unique_assets(&self) -> Vec<SimplifiedGithubAsset> {
        // First find a supported archive type
        let supported_archive = self.assets.iter().find(|v| [
            "application/x-xz",
            "application/zstd",
            "application/gzip"
        ].contains(&v.content_type.as_str())).expect("Couldn't find supported archive type");

        // Then use this archive type to find unique assets
        self.assets
            .iter()
            .filter(|a| a.content_type == supported_archive.content_type)
            .map(|f| f.clone()).collect()
    }
}

pub async fn fetch_github_releases(repo_path: &str) -> anyhow::Result<Vec<SimplifiedGithubRelease>> {
    let client = reqwest::Client::new();
    let res = client.get(format!("https://api.github.com/repos/{}/releases", repo_path))
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .header("User-Agent", "Mozilla/5.0")
        .send().await?.error_for_status();

    if res.is_err() {
        return Err(anyhow::anyhow!("Failed to fetch github releases for {} {}", repo_path, res.err().unwrap().to_string()));
    }

    Ok(res?.json::<Vec<SimplifiedGithubRelease>>().await?)
}
