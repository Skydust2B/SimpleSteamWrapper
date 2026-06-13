use std::env::temp_dir;
use std::io;
use std::path::{Path, PathBuf};
use rand::distr::{Alphanumeric, SampleString};
use tokio::fs;

pub fn get_temp_folder(prefix: &str) -> PathBuf {
    temp_dir().join(format!("{}-{}", prefix, Alphanumeric.sample_string(&mut rand::rng(), 8)))
}

async fn copy_symlink(src: &PathBuf, dst: &PathBuf) -> io::Result<()> {
    let target = fs::read_link(src).await?;
    fs::symlink(target, dst).await
}

struct CopyDirContentOpts {
    preserve_symlinks: Option<bool>
}

async fn copy_dir_content(src: &PathBuf, dst: &PathBuf, opts: &CopyDirContentOpts) -> io::Result<()> {
    fs::create_dir_all(dst).await?;

    let mut entries = fs::read_dir(src).await?;
    while let Some(entry) = entries.next_entry().await? {
        let src_path = entry.path();
        let dst_path = Path::new(dst).join(entry.file_name());

        let metadata = entry.metadata().await?;
        if opts.preserve_symlinks == Some(true) && metadata.file_type().is_symlink() {
            copy_symlink(&src_path, &dst_path).await?;
        } else if metadata.is_dir() {
            Box::pin(copy_dir_content(&src_path, &dst_path, opts)).await?;
        } else {
            fs::copy(src_path, dst_path).await?;
        }
    }

    Ok(())
}

pub async fn move_dir(src: &PathBuf, dst: &PathBuf) -> io::Result<()> {
    match fs::rename(src, dst).await {
        Ok(_) => Ok(()),
        Err(e) if e.raw_os_error() == Some(libc::EXDEV) => {
            copy_dir_content(src, dst, &CopyDirContentOpts { preserve_symlinks: Some(true) }).await?;
            fs::remove_dir_all(src).await?;
            Ok(())
        }
        Err(e) => Err(e),
    }
}

pub fn strip_all_extensions(filename: &str) -> &str {
    // First handle .tar.* specially
    if let Some(pos) = filename.rfind(".tar.") {
        return &filename[..pos];
    }

    // Otherwise, remove last extension
    if let Some((base, _ext)) = filename.rsplit_once('.') {
        return base
    }
    filename
}
