//! The only crate allowed to touch user files on disk.
//!
//! Every real operation follows: preflight → write pending operation log →
//! execute → verify → commit status (spec section 18/35). This crate only
//! owns the "execute" + "verify" part; the operation-log bookkeeping needs
//! DB access it deliberately doesn't have, so that sequencing lives in the
//! Tauri command layer that calls into this crate — see
//! apps/desktop/src-tauri/src/commands/groups.rs.

use std::path::{Path, PathBuf};

use domain::AppErrorCode;

#[derive(Debug, thiserror::Error)]
#[error("{code:?}: {message}")]
pub struct FileOpError {
    pub code: AppErrorCode,
    pub message: String,
}

impl FileOpError {
    fn new(code: AppErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

/// Windows' `ERROR_NOT_SAME_DEVICE` — returned by `rename` across volumes.
/// Windows-first per spec section 15; a portable check (`ErrorKind::CrossesDevices`)
/// can replace this if/when other platforms are supported.
const ERROR_NOT_SAME_DEVICE: i32 = 17;
/// `ERROR_SHARING_VIOLATION` / `ERROR_LOCK_VIOLATION` — file open elsewhere.
const ERROR_SHARING_VIOLATION: i32 = 32;
const ERROR_LOCK_VIOLATION: i32 = 33;

/// Picks a destination path in `dir` for `file_name` that doesn't already
/// exist, appending " (1)", " (2)", ... before the extension on collision.
/// MVP never overwrites (spec section 23.1).
pub async fn resolve_destination(dir: &Path, file_name: &str) -> PathBuf {
    resolve_free_path(dir, file_name).await
}

/// Picks a path to restore `original` to on undo: `original` itself if
/// nothing has since appeared there, otherwise "name (restored).ext",
/// "name (restored) (1).ext", ... Undo must never overwrite an unrelated
/// file that now occupies the original spot (spec section 24).
pub async fn resolve_restore_path(original: &Path) -> PathBuf {
    if tokio::fs::metadata(original).await.is_err() {
        return original.to_path_buf();
    }

    let dir = original.parent().unwrap_or_else(|| Path::new("."));
    let file_name = original
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("restored");
    let (stem, ext) = split_stem_ext(file_name);
    let restored_name = match ext {
        Some(ext) => format!("{stem} (restored).{ext}"),
        None => format!("{stem} (restored)"),
    };

    resolve_free_path(dir, &restored_name).await
}

/// Finds a free path in `dir` for `base_name`, trying it as-is first, then
/// appending " (1)", " (2)", ... before the extension.
async fn resolve_free_path(dir: &Path, base_name: &str) -> PathBuf {
    let candidate = dir.join(base_name);
    if tokio::fs::metadata(&candidate).await.is_err() {
        return candidate;
    }

    let (stem, ext) = split_stem_ext(base_name);
    for n in 1u32.. {
        let candidate_name = match ext {
            Some(ext) => format!("{stem} ({n}).{ext}"),
            None => format!("{stem} ({n})"),
        };
        let candidate = dir.join(&candidate_name);
        if tokio::fs::metadata(&candidate).await.is_err() {
            return candidate;
        }
    }
    unreachable!("collision loop is unbounded and always finds a free name")
}

fn split_stem_ext(file_name: &str) -> (&str, Option<&str>) {
    match file_name.rsplit_once('.') {
        Some((stem, ext)) if !stem.is_empty() => (stem, Some(ext)),
        _ => (file_name, None),
    }
}

/// Moves `source` to the exact `destination` (already collision-free — see
/// `resolve_destination`). Same-volume uses an atomic rename; cross-volume
/// falls back to copy → verify size → remove source (spec section 23.2).
/// Returns the final file size in bytes.
pub async fn execute_move(source: &Path, destination: &Path) -> Result<u64, FileOpError> {
    match tokio::fs::rename(source, destination).await {
        Ok(()) => tokio::fs::metadata(destination)
            .await
            .map(|m| m.len())
            .map_err(|err| classify_io_error(err, AppErrorCode::CrossVolumeMoveFailed)),
        Err(err) if err.raw_os_error() == Some(ERROR_NOT_SAME_DEVICE) => {
            copy_verify_remove(source, destination).await
        }
        Err(err) => Err(classify_io_error(err, AppErrorCode::CrossVolumeMoveFailed)),
    }
}

/// Moves `path` to the OS Recycle Bin — the only one of the four allowed
/// operations that's a terminal, non-undoable-in-app action (spec section
/// 24/25: MVP restores it via Windows' own Recycle Bin, not app-level Undo).
/// `trash::delete` is a blocking OS call, so it runs on the blocking pool
/// rather than the async runtime thread (spec section 47).
pub async fn trash(path: &Path) -> Result<(), FileOpError> {
    let path = path.to_path_buf();
    tokio::task::spawn_blocking(move || trash::delete(&path))
        .await
        .map_err(|err| FileOpError::new(AppErrorCode::InvalidPath, err.to_string()))?
        .map_err(|err| FileOpError::new(AppErrorCode::PermissionDenied, err.to_string()))
}

async fn copy_verify_remove(source: &Path, destination: &Path) -> Result<u64, FileOpError> {
    let source_len = tokio::fs::metadata(source)
        .await
        .map_err(|err| classify_io_error(err, AppErrorCode::SourceMissing))?
        .len();

    tokio::fs::copy(source, destination)
        .await
        .map_err(|err| classify_io_error(err, AppErrorCode::CrossVolumeMoveFailed))?;

    let dest_len = tokio::fs::metadata(destination)
        .await
        .map_err(|err| classify_io_error(err, AppErrorCode::CrossVolumeMoveFailed))?
        .len();

    if dest_len != source_len {
        let _ = tokio::fs::remove_file(destination).await;
        return Err(FileOpError::new(
            AppErrorCode::CrossVolumeMoveFailed,
            format!("copied {dest_len} bytes but source was {source_len} bytes"),
        ));
    }

    // Only remove the source once the copy is verified — spec 23.2: the
    // source must survive a failed cross-volume move.
    tokio::fs::remove_file(source)
        .await
        .map_err(|err| classify_io_error(err, AppErrorCode::CrossVolumeMoveFailed))?;

    Ok(dest_len)
}

fn classify_io_error(err: std::io::Error, default: AppErrorCode) -> FileOpError {
    let code = match err.kind() {
        std::io::ErrorKind::NotFound => AppErrorCode::SourceMissing,
        std::io::ErrorKind::PermissionDenied => AppErrorCode::PermissionDenied,
        _ if matches!(
            err.raw_os_error(),
            Some(ERROR_SHARING_VIOLATION) | Some(ERROR_LOCK_VIOLATION)
        ) =>
        {
            AppErrorCode::FileLocked
        }
        _ => default,
    };
    FileOpError::new(code, err.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unique_dir(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "file-operations-test-{}-{}",
            std::process::id(),
            name
        ))
    }

    #[tokio::test]
    async fn resolve_destination_picks_free_name_on_collision() {
        let dir = unique_dir("collisions");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("a.png"), b"1").unwrap();
        std::fs::write(dir.join("a (1).png"), b"1").unwrap();

        let resolved = resolve_destination(&dir, "a.png").await;

        std::fs::remove_dir_all(&dir).ok();
        assert_eq!(resolved.file_name().unwrap().to_str().unwrap(), "a (2).png");
    }

    #[tokio::test]
    async fn resolve_destination_keeps_original_name_when_free() {
        let dir = unique_dir("free");
        std::fs::create_dir_all(&dir).unwrap();

        let resolved = resolve_destination(&dir, "b.png").await;

        std::fs::remove_dir_all(&dir).ok();
        assert_eq!(resolved.file_name().unwrap().to_str().unwrap(), "b.png");
    }

    #[tokio::test]
    async fn resolve_restore_path_reuses_original_when_free() {
        let dir = unique_dir("restore-free");
        std::fs::create_dir_all(&dir).unwrap();
        let original = dir.join("a.png");

        let resolved = resolve_restore_path(&original).await;

        std::fs::remove_dir_all(&dir).ok();
        assert_eq!(resolved, original);
    }

    #[tokio::test]
    async fn resolve_restore_path_avoids_overwriting_unrelated_file() {
        let dir = unique_dir("restore-conflict");
        std::fs::create_dir_all(&dir).unwrap();
        let original = dir.join("a.png");
        // Something else now occupies the original spot.
        std::fs::write(&original, b"unrelated file").unwrap();

        let resolved = resolve_restore_path(&original).await;

        std::fs::remove_dir_all(&dir).ok();
        assert_eq!(
            resolved.file_name().unwrap().to_str().unwrap(),
            "a (restored).png"
        );
    }

    #[tokio::test]
    async fn execute_move_moves_file_within_same_volume() {
        let dir = unique_dir("move-same-volume");
        std::fs::create_dir_all(&dir).unwrap();
        let source = dir.join("source.txt");
        let destination = dir.join("dest.txt");
        std::fs::write(&source, b"hello world").unwrap();

        let size = execute_move(&source, &destination).await.unwrap();

        assert_eq!(size, 11);
        assert!(!source.exists());
        assert!(destination.exists());
        std::fs::remove_dir_all(&dir).ok();
    }

    #[tokio::test]
    async fn execute_move_reports_source_missing() {
        let dir = unique_dir("move-missing");
        std::fs::create_dir_all(&dir).unwrap();
        let source = dir.join("does-not-exist.txt");
        let destination = dir.join("dest.txt");

        let err = execute_move(&source, &destination).await.unwrap_err();

        std::fs::remove_dir_all(&dir).ok();
        assert!(matches!(err.code, AppErrorCode::SourceMissing));
    }
}
