use std::io;
use std::path::{PathBuf};
use std::sync::Arc;
use autocompress::autodetect_async_buf_reader;
use autocompress::xz::{AsyncXzDecompressReader};
use futures_util::TryStreamExt;
use log::{error, info, warn};
use tokio::fs;
use tokio::io::AsyncRead;
use tokio_tar::Archive;
use tokio_util::io::StreamReader;
use crate::dl_manager::downloadable_asset::DownloadableAsset;
use crate::steam::steam::{create_compatibility_tool_vdf, get_steam_compat_tools_path};
use crate::utils::io_utils::{get_temp_folder, move_dir};

type ProgressCallback = Arc<dyn Fn(u64, u64) + Send + Sync>;

pub async fn download_and_extract_release_internal(
    asset: &DownloadableAsset,
    on_progress: ProgressCallback,
    temp_folder_path: &PathBuf
) -> anyhow::Result<()> {
    info!("Downloading asset {:?}", asset);

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

    if let Some(entry) = files.next_entry().await? {
        let destination = if asset.custom_folder.is_some() {
            compat.join(asset.custom_folder.clone().unwrap())
        } else {
            compat.join(entry.file_name())
        };
        info!("Moving {} to {}", entry.path().display(), destination.display());
        if destination.exists() {
            warn!("Destination exists, removing it...");
            fs::remove_dir_all(&destination).await?
        }
        move_dir(&entry.path(), &destination).await?;
        if asset.custom_folder.is_some()  {
            info!("Custom destination, writing version file...");
            fs::create_dir_all(destination.clone()).await?;
            fs::write(destination.clone().join("ssw_ct_version"), asset.asset_name.clone()).await?;

            info!("Replacing compatibilitytool.vdf...");
            let dest_clone = destination.clone();
            let name = dest_clone.file_name().clone().unwrap().to_str().unwrap();
            let comp_vdf_file = create_compatibility_tool_vdf(name, name);
            fs::write(destination.clone().join("compatibilitytool.vdf"), comp_vdf_file).await?;
        }
    }
    Ok(())
}

pub async fn download_and_extract_asset(
    asset: &DownloadableAsset,
    on_progress: ProgressCallback,
) -> anyhow::Result<()> {
    let temp_folder_path = get_temp_folder("ssw");

    fs::create_dir_all(&temp_folder_path).await?;
    let result = download_and_extract_release_internal(asset, on_progress, &temp_folder_path).await;
    if result.is_err() {
        error!("{:?}", result.err().unwrap());
    }
    fs::remove_dir_all(temp_folder_path).await?;
    Ok(())
}
