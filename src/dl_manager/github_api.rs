use serde::{Deserialize, Serialize};

#[derive(Debug,Serialize,Deserialize)]
pub struct SimplifiedGithubAsset {
    pub id: usize,
    pub name: String,
    pub browser_download_url: String,
    pub(crate) content_type: String, // 	"application/x-xz"(.tar.xz) for cachyos and "application/zstd"/"application/gzip" (.tar.zst/.tar.gz)
    pub created_at: String
}

#[derive(Debug,Serialize,Deserialize)]
pub struct SimplifiedGithubRelease {
    pub id: usize,
    pub name: String,
    pub published_at: String,
    pub(crate) assets: Vec<SimplifiedGithubAsset>
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
