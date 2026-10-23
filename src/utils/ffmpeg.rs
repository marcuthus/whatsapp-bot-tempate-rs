use std::{
    fs, io,
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicU32, Ordering},
};

use thiserror::Error;

static NEXT_DIRECTORY: AtomicU32 = AtomicU32::new(0);

#[derive(Debug, Error)]
pub enum FfmpegError {
    #[error("o ffmpeg não está instalado no servidor")]
    Missing,
    #[error("o ffmpeg falhou: {0}")]
    Failed(String),
}

pub struct TempDirectory(PathBuf);

impl TempDirectory {
    pub fn create() -> io::Result<Self> {
        let path = std::env::temp_dir().join(format!(
            "media-{}-{}",
            std::process::id(),
            NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed)
        ));

        fs::create_dir_all(&path)?;

        Ok(Self(path))
    }

    pub fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn binary() -> String {
    std::env::var("FFMPEG_PATH").unwrap_or_else(|_| "ffmpeg".to_owned())
}

#[cfg(test)]
pub fn is_available() -> bool {
    Command::new(binary()).arg("-version").output().is_ok()
}

pub fn run(directory: &Path, args: &[&str]) -> Result<(), FfmpegError> {
    let output = Command::new(binary())
        .current_dir(directory)
        .args(["-y", "-hide_banner", "-loglevel", "error"])
        .args(args)
        .output()
        .map_err(|error| match error.kind() {
            io::ErrorKind::NotFound => FfmpegError::Missing,
            _ => FfmpegError::Failed(error.to_string()),
        })?;

    if output.status.success() {
        Ok(())
    } else {
        Err(FfmpegError::Failed(
            String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_distinct_directories_that_are_removed_on_drop() {
        let first = TempDirectory::create().unwrap();
        let second = TempDirectory::create().unwrap();
        let removed = first.path().to_owned();

        assert_ne!(first.path(), second.path());
        assert!(removed.exists());

        drop(first);

        assert!(!removed.exists());
    }

    #[test]
    fn reports_a_missing_binary() {
        let directory = TempDirectory::create().unwrap();

        let result = Command::new("ffmpeg-binary-that-does-not-exist")
            .current_dir(directory.path())
            .output()
            .map_err(|error| error.kind());

        assert_eq!(result.unwrap_err(), io::ErrorKind::NotFound);
    }
}
