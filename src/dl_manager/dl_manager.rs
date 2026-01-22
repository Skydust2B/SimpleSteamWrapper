use std::env::temp_dir;
use std::future::Future;
use std::io;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::sync::Arc;
use autocompress::autodetect_async_buf_reader;
use autocompress::xz::{AsyncXzDecompressReader};
use fs_extra::dir::CopyOptions;
use futures_util::TryStreamExt;
use log::{error, info};
use rand::distr::{Alphanumeric, SampleString};
use tokio::fs;
use tokio::io::AsyncRead;
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

async fn move_dir(src: &PathBuf, dst: &PathBuf) -> io::Result<()> {
    match fs::rename(src, dst).await {
        Ok(_) => Ok(()),
        Err(e) if e.raw_os_error() == Some(libc::EXDEV) => {
            fs::create_dir_all(dst).await?;
            let _ = tokio::task::spawn_blocking({
                let src = src.clone();
                let dst = dst.clone();
                move || {
                    fs_extra::dir::copy(&src, &dst, &CopyOptions::new().content_only(true)).unwrap();
                }
            }).await?;
            fs::remove_dir_all(src).await?;
            Ok(())
        }
        Err(e) => Err(e),
    }
}

type ProgressCallback = Arc<dyn Fn(u64, u64) + Send + Sync>;

pub async fn download_and_extract_release_internal(
    release: &SimplifiedGithubRelease,
    on_progress: ProgressCallback,
    temp_folder_path: &PathBuf
) -> anyhow::Result<()> {
    let asset = find_first_supported_archive(release).ok_or(anyhow::anyhow!("No supported archive found"))?;
    info!("Downloading release {:?}", release);

    info!("Creating temp folder {}", temp_folder_path.display());
    fs::create_dir_all(&temp_folder_path).await?;

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
        .map_err(|err| io::Error::new(io::ErrorKind::Other, err));

    info!("Downloading and extracting {} bytes", total_size);

    let stream_reader = StreamReader::new(stream);
    let reader: Box<dyn AsyncRead + Unpin + Send> =
        if asset.content_type == "application/x-xz" {
            Box::new(AsyncXzDecompressReader::new(stream_reader))
        } else {
            Box::new(autodetect_async_buf_reader(stream_reader).await?)
        };

    Archive::new(reader).unpack(&temp_folder_path).await?;

    let compat = get_steam_compat_tools_path();
    info!("Moving downloaded release to {}", compat.display());
    let mut files = fs::read_dir(temp_folder_path).await?;

    while let Some(entry) = files.next_entry().await? {
        info!("Moving {} to {}", entry.path().display(), compat.join(entry.file_name()).display());
        move_dir(&entry.path(), &compat.join(entry.file_name())).await?;
    }
    Ok(())
}

pub async fn download_and_extract_release(
    release: &SimplifiedGithubRelease,
    on_progress: ProgressCallback,
) -> anyhow::Result<()> {
    let temp_folder_path = get_temp_folder("ssw");

    fs::create_dir_all(&temp_folder_path).await?;
    let result = download_and_extract_release_internal(release, on_progress, &temp_folder_path).await;
    if result.is_err() {
        error!("{:?}", result.err().unwrap());
    }
    fs::remove_dir_all(temp_folder_path).await?;
    Ok(())
}
