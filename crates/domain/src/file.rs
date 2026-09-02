use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::AppErrorCode;

/// Full lifecycle of a tracked file. See docs/download_inbox_product_technical_spec_v0.2.md
/// section 13 for the state machine this enum implements.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileStatus {
    Detected,
    WaitingStable,
    PendingRetry,
    Ready,
    Pending,
    Organizing,
    Organized,
    Temporary,
    Expired,
    CleanupReady,
    Later,
    Trashed,
    Restoring,
    Error,
    Missing,
}

impl FileStatus {
    /// Stable string form used as the `files.status` column value — kept
    /// separate from `Display`/serde so storage doesn't need a JSON round
    /// trip just to bind a SQLite TEXT column.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Detected => "detected",
            Self::WaitingStable => "waiting_stable",
            Self::PendingRetry => "pending_retry",
            Self::Ready => "ready",
            Self::Pending => "pending",
            Self::Organizing => "organizing",
            Self::Organized => "organized",
            Self::Temporary => "temporary",
            Self::Expired => "expired",
            Self::CleanupReady => "cleanup_ready",
            Self::Later => "later",
            Self::Trashed => "trashed",
            Self::Restoring => "restoring",
            Self::Error => "error",
            Self::Missing => "missing",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "detected" => Self::Detected,
            "waiting_stable" => Self::WaitingStable,
            "pending_retry" => Self::PendingRetry,
            "ready" => Self::Ready,
            "pending" => Self::Pending,
            "organizing" => Self::Organizing,
            "organized" => Self::Organized,
            "temporary" => Self::Temporary,
            "expired" => Self::Expired,
            "cleanup_ready" => Self::CleanupReady,
            "later" => Self::Later,
            "trashed" => Self::Trashed,
            "restoring" => Self::Restoring,
            "error" => Self::Error,
            "missing" => Self::Missing,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileRecord {
    pub id: Uuid,
    pub original_name: String,
    pub current_name: String,
    pub original_path: String,
    pub current_path: String,
    pub extension: Option<String>,
    pub mime_type: Option<String>,
    pub size_bytes: Option<i64>,
    pub status: FileStatus,
    pub detected_at: DateTime<Utc>,
    pub ready_at: Option<DateTime<Utc>>,
    pub organized_at: Option<DateTime<Utc>>,
    pub last_seen_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub group_id: Option<Uuid>,
    pub source_context_id: Option<Uuid>,
    pub error_code: Option<AppErrorCode>,
    pub error_message: Option<String>,
}
