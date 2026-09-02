//! Answers one question: is this path a finished, operable file yet?
//!
//! Not the watcher's job — `file-watcher` just reports that *something*
//! happened at a path. This crate owns the stability-check algorithm from
//! spec section 20: don't trust a `Create` event, poll size/mtime until they
//! stop changing, then confirm the file is actually shareable-readable
//! (not still held open exclusively by the downloader).

use std::path::Path;
use std::time::{Duration, Instant, SystemTime};

use chrono::{DateTime, Utc};

/// Extensions browsers use for in-progress downloads. A file under one of
/// these is never itself a candidate — see spec section 20.1: the rename
/// away from this extension is the real signal, not this file's contents.
const TEMP_EXTENSIONS: &[&str] = &["crdownload", "part", "tmp", "download"];

pub fn is_temp_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| TEMP_EXTENSIONS.iter().any(|t| t.eq_ignore_ascii_case(ext)))
        .unwrap_or(false)
}

#[derive(Debug, Clone, Copy)]
pub struct StabilityConfig {
    pub check_interval: Duration,
    pub stable_rounds: u32,
    pub max_wait: Duration,
}

impl Default for StabilityConfig {
    /// Matches the recommended defaults in spec section 20.2.
    fn default() -> Self {
        Self {
            check_interval: Duration::from_millis(300),
            stable_rounds: 2,
            max_wait: Duration::from_secs(120),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ReadyFile {
    pub size_bytes: u64,
    pub modified_at: DateTime<Utc>,
}

#[derive(Debug, thiserror::Error)]
pub enum DetectorError {
    #[error("source file went missing while waiting for it to finish downloading")]
    SourceMissing,
    #[error("permission denied while checking download stability")]
    PermissionDenied,
    #[error("timed out waiting for the file to become stable")]
    Timeout,
}

/// Polls `path` until its size/mtime hold steady for `stable_rounds` checks
/// and it can be opened for a shared read (i.e. the downloader isn't still
/// holding an exclusive lock on it), or until `max_wait` elapses.
///
/// Runs as its own async task per candidate path — a slow, multi-GB download
/// must never block the watcher or other candidates (spec section 20.2).
pub async fn wait_until_ready(
    path: &Path,
    config: &StabilityConfig,
) -> Result<ReadyFile, DetectorError> {
    let start = Instant::now();
    let mut last: Option<(u64, SystemTime)> = None;
    let mut stable_count = 0u32;

    loop {
        if start.elapsed() > config.max_wait {
            return Err(DetectorError::Timeout);
        }

        let metadata = match tokio::fs::metadata(path).await {
            Ok(metadata) => metadata,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                return Err(DetectorError::SourceMissing);
            }
            Err(err) if err.kind() == std::io::ErrorKind::PermissionDenied => {
                return Err(DetectorError::PermissionDenied);
            }
            Err(err) => {
                tracing::debug!(?err, path = %path.display(), "transient metadata read failure, retrying");
                stable_count = 0;
                tokio::time::sleep(config.check_interval).await;
                continue;
            }
        };

        let current = (
            metadata.len(),
            metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH),
        );

        if last == Some(current) {
            stable_count += 1;
        } else {
            stable_count = 0;
            last = Some(current);
        }

        if stable_count >= config.stable_rounds {
            if tokio::fs::File::open(path).await.is_ok() {
                return Ok(ReadyFile {
                    size_bytes: current.0,
                    modified_at: DateTime::<Utc>::from(current.1),
                });
            }
            // Still exclusively locked by the writer — don't trust the
            // stability streak, keep waiting.
            stable_count = 0;
        }

        tokio::time::sleep(config.check_interval).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn temp_extensions_are_ignored_case_insensitively() {
        assert!(is_temp_extension(Path::new("a.crdownload")));
        assert!(is_temp_extension(Path::new("a.part")));
        assert!(is_temp_extension(Path::new("a.tmp")));
        assert!(is_temp_extension(Path::new("a.download")));
        assert!(is_temp_extension(Path::new("a.CrDownload")));
    }

    #[test]
    fn final_names_are_not_temp_extensions() {
        assert!(!is_temp_extension(Path::new("a.png")));
        assert!(!is_temp_extension(Path::new("a.pdf")));
        assert!(!is_temp_extension(Path::new("no-extension")));
    }

    fn unique_temp_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "download-detector-test-{}-{}",
            std::process::id(),
            name
        ))
    }

    #[tokio::test]
    async fn ready_file_is_reported_once_stable() {
        let path = unique_temp_path("stable.txt");
        std::fs::write(&path, b"hello").unwrap();

        let config = StabilityConfig {
            check_interval: Duration::from_millis(10),
            stable_rounds: 2,
            max_wait: Duration::from_secs(5),
        };

        let result = wait_until_ready(&path, &config).await;
        std::fs::remove_file(&path).ok();

        let ready = result.expect("a file that never changes should become ready");
        assert_eq!(ready.size_bytes, 5);
    }

    #[tokio::test]
    async fn missing_file_reports_source_missing() {
        let path = unique_temp_path("does-not-exist.txt");
        let config = StabilityConfig {
            check_interval: Duration::from_millis(10),
            stable_rounds: 2,
            max_wait: Duration::from_secs(5),
        };

        let err = wait_until_ready(&path, &config).await.unwrap_err();
        assert!(matches!(err, DetectorError::SourceMissing));
    }
}
