use std::env::temp_dir;
use std::path::PathBuf;
use std::sync::Arc;
use autocompress::autodetect_async_buf_reader;
use futures_util::TryStreamExt;
use log::info;
use rand::distr::{Alphanumeric, SampleString};
use tokio::fs;
use tokio_tar::Archive;
use tokio_util::io::StreamReader;
use crate::compatibility_tools::steam::get_steam_compat_tools_path;
use crate::dl_manager::github_api::{SimplifiedGithubAsset, SimplifiedGithubRelease};

pub fn find_first_supported_archive(release: &SimplifiedGithubRelease) -> Option<&SimplifiedGithubAsset> {
    release.assets.iter().find(|v| [
        "application/x-xz",
        "application/zstd",
        "application/gzip"
    ].contains(&v.content_type.as_str()))
}

pub fn get_temp_folder(prefix: &str) -> PathBuf {
    temp_dir().join(format!("{}-{}", prefix, Alphanumeric.sample_string(&mut rand::rng(), 8)))
}

type ProgressCallback = Arc<dyn Fn(u64, u64) + Send + Sync>;

pub async fn download_and_extract_release(
    release: &SimplifiedGithubRelease,
    on_progress: ProgressCallback,
) -> anyhow::Result<()> {
    let asset = find_first_supported_archive(release).ok_or(anyhow::anyhow!("No supported archive found"))?;

    let temp_folder_path = get_temp_folder("ssw");
    let temp_extract_path = temp_folder_path.join("extracted");

    info!("Creating temp folder {}", temp_extract_path.display());
    fs::create_dir_all(&temp_extract_path).await?;

    let client = reqwest::Client::new();
    let retrieved_file = client.get(asset.browser_download_url.clone())
        .header("User-Agent", "Mozilla/5.0")
        .send().await?.error_for_status()?;

    let total_size = retrieved_file.content_length().unwrap_or(0);
    let mut downloaded = 0u64;
    
    let stream = retrieved_file
        .bytes_stream()
        .inspect_ok(|chunk| {
            downloaded += chunk.len() as u64;
            let on_progress = on_progress.clone();

            slint::invoke_from_event_loop(move || {
                on_progress(downloaded, total_size);
            }).ok();
        })
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err));

    info!("Downloading and extracting {} bytes", total_size);
    let stream_reader = StreamReader::new(stream);
    let reader = autodetect_async_buf_reader(stream_reader).await?;
    Archive::new(reader).unpack(temp_extract_path.clone()).await?;
    let compat = get_steam_compat_tools_path();

    info!("Moving downloaded release to {}", compat.display());
    let mut files = fs::read_dir(temp_extract_path).await?;
    while let Some(entry) = files.next_entry().await? {
        fs::rename(entry.path(), compat.join(entry.file_name())).await?;
    }
    Ok(())
}
