use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::AppErrorCode;

/// The only kinds of filesystem action the app is ever allowed to perform.
/// Every mutation must go through the `file-operations` crate and produce
/// one of these as an audit / undo record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationType {
    Move,
    Rename,
    Restore,
    Trash,
}

impl OperationType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Move => "move",
            Self::Rename => "rename",
            Self::Restore => "restore",
            Self::Trash => "trash",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "move" => Self::Move,
            "rename" => Self::Rename,
            "restore" => Self::Restore,
            "trash" => Self::Trash,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationStatus {
    Pending,
    Completed,
    Failed,
    Undone,
}

impl OperationStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Undone => "undone",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "pending" => Self::Pending,
            "completed" => Self::Completed,
            "failed" => Self::Failed,
            "undone" => Self::Undone,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operation {
    pub id: Uuid,
    pub file_id: Uuid,
    pub operation_type: OperationType,
    pub source_path: Option<String>,
    pub destination_path: Option<String>,
    /// Which group a `Move` was filing into, if any — lets crash
    /// reconciliation (spec section 39) restore the file's group assignment
    /// after an interrupted move, not just its path.
    pub group_id: Option<Uuid>,
    pub status: OperationStatus,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub undone_at: Option<DateTime<Utc>>,
    pub error_code: Option<AppErrorCode>,
    pub error_message: Option<String>,
}
