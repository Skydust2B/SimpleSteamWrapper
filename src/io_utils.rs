use std::env::temp_dir;
use std::io;
use std::path::PathBuf;
use fs_extra::dir::CopyOptions;
use rand::distr::{Alphanumeric, SampleString};
use tokio::fs;

pub fn get_temp_folder(prefix: &str) -> PathBuf {
    temp_dir().join(format!("{}-{}", prefix, Alphanumeric.sample_string(&mut rand::rng(), 8)))
}

pub async fn move_dir(src: &PathBuf, dst: &PathBuf) -> io::Result<()> {
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
