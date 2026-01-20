use serde::{Deserialize, Serialize};

#[derive(Debug,Serialize,Deserialize)]
pub struct SimplifiedGithubAsset {
    id: usize,
    name: String,
    url: String,
    content_type: String,
    created_at: String
}

#[derive(Debug,Serialize,Deserialize)]
pub struct SimplifiedGithubRelease {
    id: usize,
    name: String,
    published_at: String,
    assets: Vec<SimplifiedGithubAsset>
}

pub async fn fetch_github_releases(repo_path: &str) -> anyhow::Result<Vec<SimplifiedGithubRelease>> {
    let client = reqwest::Client::new();
    let res = client.get(format!("https://api.github.com/repos/{}/releases", repo_path))
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
        .header("User-Agent", "Mozilla/5.0")
        .send().await;

    if res.is_err() {
        return Err(anyhow::anyhow!("Failed to fetch github releases for {} {}", repo_path, res.err().unwrap().to_string()));
    }

    Ok(res?.json::<Vec<SimplifiedGithubRelease>>().await?)
}
