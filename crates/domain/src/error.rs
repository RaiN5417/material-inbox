use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Machine-readable error codes surfaced to the UI. Never expose raw OS
/// error text to the user — map it to one of these first.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AppErrorCode {
    PermissionDenied,
    SourceMissing,
    DestinationExists,
    FileLocked,
    InvalidPath,
    DownloadNotComplete,
    DatabaseError,
    WatcherError,
    CrossVolumeMoveFailed,
    UndoConflict,
}

impl AppErrorCode {
    /// Stable string form used for the `*.error_code` columns — see
    /// `FileStatus::as_str` for why this isn't routed through serde.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PermissionDenied => "permission_denied",
            Self::SourceMissing => "source_missing",
            Self::DestinationExists => "destination_exists",
            Self::FileLocked => "file_locked",
            Self::InvalidPath => "invalid_path",
            Self::DownloadNotComplete => "download_not_complete",
            Self::DatabaseError => "database_error",
            Self::WatcherError => "watcher_error",
            Self::CrossVolumeMoveFailed => "cross_volume_move_failed",
            Self::UndoConflict => "undo_conflict",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "permission_denied" => Self::PermissionDenied,
            "source_missing" => Self::SourceMissing,
            "destination_exists" => Self::DestinationExists,
            "file_locked" => Self::FileLocked,
            "invalid_path" => Self::InvalidPath,
            "download_not_complete" => Self::DownloadNotComplete,
            "database_error" => Self::DatabaseError,
            "watcher_error" => Self::WatcherError,
            "cross_volume_move_failed" => Self::CrossVolumeMoveFailed,
            "undo_conflict" => Self::UndoConflict,
            _ => return None,
        })
    }
}

#[derive(Debug, Error)]
#[error("{code:?}: {message}")]
pub struct DomainError {
    pub code: AppErrorCode,
    pub message: String,
}

impl DomainError {
    pub fn new(code: AppErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}
