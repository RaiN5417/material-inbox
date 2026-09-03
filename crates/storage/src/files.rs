use chrono::{DateTime, Utc};
use domain::{AppErrorCode, FileRecord, FileStatus};
use uuid::Uuid;

use crate::{DbPool, StorageError};

const FILE_COLUMNS: &str = "id, original_name, current_name, original_path, current_path, \
     extension, mime_type, size_bytes, status, detected_at, ready_at, organized_at, \
     last_seen_at, expires_at, group_id, source_context_id, error_code, error_message";

/// Inserts a newly detected file (spec section 13: starts life as `Detected`).
pub async fn insert_file(pool: &DbPool, file: &FileRecord) -> Result<(), StorageError> {
    sqlx::query(
        "INSERT INTO files (
            id, original_name, current_name, original_path, current_path,
            extension, mime_type, size_bytes, status, detected_at, ready_at,
            organized_at, last_seen_at, expires_at, group_id, source_context_id,
            error_code, error_message
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(file.id.to_string())
    .bind(&file.original_name)
    .bind(&file.current_name)
    .bind(&file.original_path)
    .bind(&file.current_path)
    .bind(&file.extension)
    .bind(&file.mime_type)
    .bind(file.size_bytes)
    .bind(file.status.as_str())
    .bind(file.detected_at.to_rfc3339())
    .bind(file.ready_at.map(|t| t.to_rfc3339()))
    .bind(file.organized_at.map(|t| t.to_rfc3339()))
    .bind(file.last_seen_at.to_rfc3339())
    .bind(file.expires_at.map(|t| t.to_rfc3339()))
    .bind(file.group_id.map(|id| id.to_string()))
    .bind(file.source_context_id.map(|id| id.to_string()))
    .bind(file.error_code.map(|c| c.as_str()))
    .bind(&file.error_message)
    .execute(pool)
    .await?;

    Ok(())
}

/// Stability check succeeded: the file is a finished, operable download.
/// Moves it straight from `Detected`/`WaitingStable` to `Pending` (spec
/// section 13 treats `Ready` as a momentary, non-persisted waypoint) and
/// refreshes the fields the detector only knows once the file settles.
#[allow(clippy::too_many_arguments)]
pub async fn mark_ready_pending(
    pool: &DbPool,
    id: Uuid,
    current_name: &str,
    current_path: &str,
    size_bytes: i64,
    ready_at: DateTime<Utc>,
) -> Result<(), StorageError> {
    sqlx::query(
        "UPDATE files SET
            status = ?, current_name = ?, current_path = ?, size_bytes = ?,
            ready_at = ?, last_seen_at = ?, error_code = NULL, error_message = NULL
         WHERE id = ?",
    )
    .bind(FileStatus::Pending.as_str())
    .bind(current_name)
    .bind(current_path)
    .bind(size_bytes)
    .bind(ready_at.to_rfc3339())
    .bind(Utc::now().to_rfc3339())
    .bind(id.to_string())
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_file(pool: &DbPool, id: Uuid) -> Result<Option<FileRecord>, StorageError> {
    let row =
        sqlx::query_as::<_, FileRow>(&format!("SELECT {FILE_COLUMNS} FROM files WHERE id = ?"))
            .bind(id.to_string())
            .fetch_optional(pool)
            .await?;

    Ok(row.map(Into::into))
}

/// Looks up whichever tracked file currently sits at `path` — the watcher's
/// external-deletion detection has nothing but the path a Remove event fired
/// on to go by. Most-recently-seen wins on the rare chance more than one row
/// somehow shares a path.
pub async fn find_file_by_path(
    pool: &DbPool,
    path: &str,
) -> Result<Option<FileRecord>, StorageError> {
    let row = sqlx::query_as::<_, FileRow>(&format!(
        "SELECT {FILE_COLUMNS} FROM files WHERE current_path = ? ORDER BY last_seen_at DESC LIMIT 1"
    ))
    .bind(path)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(Into::into))
}

/// Commits a successful move into a group (spec section 23): file now lives
/// at `current_path` under `group_id`, status `Organized`.
pub async fn assign_group(
    pool: &DbPool,
    file_id: Uuid,
    group_id: Uuid,
    current_name: &str,
    current_path: &str,
) -> Result<(), StorageError> {
    sqlx::query(
        "UPDATE files SET
            status = ?, group_id = ?, current_name = ?, current_path = ?,
            organized_at = ?, last_seen_at = ?, error_code = NULL, error_message = NULL
         WHERE id = ?",
    )
    .bind(FileStatus::Organized.as_str())
    .bind(group_id.to_string())
    .bind(current_name)
    .bind(current_path)
    .bind(Utc::now().to_rfc3339())
    .bind(Utc::now().to_rfc3339())
    .bind(file_id.to_string())
    .execute(pool)
    .await?;

    Ok(())
}

/// Renames a file in place (same directory, new name) — doesn't touch
/// `status`/`group_id`, unlike `assign_group`.
pub async fn rename_file(
    pool: &DbPool,
    file_id: Uuid,
    current_name: &str,
    current_path: &str,
) -> Result<(), StorageError> {
    sqlx::query(
        "UPDATE files SET current_name = ?, current_path = ?, last_seen_at = ? WHERE id = ?",
    )
    .bind(current_name)
    .bind(current_path)
    .bind(Utc::now().to_rfc3339())
    .bind(file_id.to_string())
    .execute(pool)
    .await?;

    Ok(())
}

/// Undo of a move (spec section 24/13): file is back at `current_path`, no
/// longer belongs to a group, status `Pending` again (same waypoint a
/// freshly-detected file lands on — the user makes a fresh decision).
pub async fn mark_restored(
    pool: &DbPool,
    file_id: Uuid,
    current_name: &str,
    current_path: &str,
) -> Result<(), StorageError> {
    sqlx::query(
        "UPDATE files SET
            status = ?, group_id = NULL, current_name = ?, current_path = ?,
            organized_at = NULL, last_seen_at = ?, error_code = NULL, error_message = NULL
         WHERE id = ?",
    )
    .bind(FileStatus::Pending.as_str())
    .bind(current_name)
    .bind(current_path)
    .bind(Utc::now().to_rfc3339())
    .bind(file_id.to_string())
    .execute(pool)
    .await?;

    Ok(())
}

/// Marks (or re-marks, for "Keep N more days") a file `Temporary` with a
/// fresh `expires_at` (spec section 25).
pub async fn mark_temporary(
    pool: &DbPool,
    file_id: Uuid,
    expires_at: DateTime<Utc>,
) -> Result<(), StorageError> {
    sqlx::query(
        "UPDATE files SET
            status = ?, expires_at = ?, last_seen_at = ?, error_code = NULL, error_message = NULL
         WHERE id = ?",
    )
    .bind(FileStatus::Temporary.as_str())
    .bind(expires_at.to_rfc3339())
    .bind(Utc::now().to_rfc3339())
    .bind(file_id.to_string())
    .execute(pool)
    .await?;

    Ok(())
}

/// Files the Inbox shows: pending a decision, deferred to later, or stuck in
/// an error/retry state (spec section 11: "Pending, Later, Failed
/// operations"). Loaded on startup so the Inbox survives a restart instead
/// of only reflecting the current session's live events.
pub async fn list_inbox(pool: &DbPool) -> Result<Vec<FileRecord>, StorageError> {
    let rows = sqlx::query_as::<_, FileRow>(&format!(
        "SELECT {FILE_COLUMNS} FROM files WHERE status IN (?, ?, ?, ?) ORDER BY detected_at DESC"
    ))
    .bind(FileStatus::Pending.as_str())
    .bind(FileStatus::Later.as_str())
    .bind(FileStatus::Error.as_str())
    .bind(FileStatus::PendingRetry.as_str())
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(Into::into).collect())
}

/// Files currently filed under a group — what the sidebar's folder view and
/// the Groups panel show once a specific group is selected (spec section 23
/// never persisted a listing for this; it's a plain filter on `group_id`).
pub async fn list_files_by_group(
    pool: &DbPool,
    group_id: Uuid,
) -> Result<Vec<FileRecord>, StorageError> {
    let rows = sqlx::query_as::<_, FileRow>(&format!(
        "SELECT {FILE_COLUMNS} FROM files WHERE group_id = ? AND status != ? ORDER BY organized_at DESC"
    ))
    .bind(group_id.to_string())
    .bind(FileStatus::Trashed.as_str())
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(Into::into).collect())
}

/// Files the Temporary panel shows: still counting down, or expired and
/// waiting on a decision (spec section 11).
pub async fn list_temporary(pool: &DbPool) -> Result<Vec<FileRecord>, StorageError> {
    let rows = sqlx::query_as::<_, FileRow>(&format!(
        "SELECT {FILE_COLUMNS} FROM files WHERE status IN (?, ?) ORDER BY expires_at ASC"
    ))
    .bind(FileStatus::Temporary.as_str())
    .bind(FileStatus::CleanupReady.as_str())
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(Into::into).collect())
}

/// Promotes every `Temporary` file whose `expires_at` has passed to
/// `CleanupReady` (spec section 13 collapses the momentary `Expired`
/// waypoint into this single transition, the same way `Ready` never gets
/// persisted on its own). Returns the files that just changed, so the
/// caller can notify the UI.
pub async fn sweep_expired(pool: &DbPool) -> Result<Vec<FileRecord>, StorageError> {
    let now = Utc::now().to_rfc3339();
    let rows = sqlx::query_as::<_, FileRow>(&format!(
        "UPDATE files SET status = ?, last_seen_at = ?
         WHERE status = ? AND expires_at IS NOT NULL AND expires_at <= ?
         RETURNING {FILE_COLUMNS}"
    ))
    .bind(FileStatus::CleanupReady.as_str())
    .bind(&now)
    .bind(FileStatus::Temporary.as_str())
    .bind(&now)
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(Into::into).collect())
}

/// Moved to the OS Recycle Bin (spec section 25): terminal state, no
/// in-app Undo — the user can still restore it from Windows' own Recycle
/// Bin.
pub async fn mark_trashed(pool: &DbPool, file_id: Uuid) -> Result<(), StorageError> {
    sqlx::query("UPDATE files SET status = ?, last_seen_at = ? WHERE id = ?")
        .bind(FileStatus::Trashed.as_str())
        .bind(Utc::now().to_rfc3339())
        .bind(file_id.to_string())
        .execute(pool)
        .await?;

    Ok(())
}

/// Records a plain status transition (e.g. into `PendingRetry` or `Error`).
pub async fn mark_status(
    pool: &DbPool,
    id: Uuid,
    status: FileStatus,
    error_code: Option<AppErrorCode>,
    error_message: Option<&str>,
) -> Result<(), StorageError> {
    sqlx::query(
        "UPDATE files SET status = ?, last_seen_at = ?, error_code = ?, error_message = ?
         WHERE id = ?",
    )
    .bind(status.as_str())
    .bind(Utc::now().to_rfc3339())
    .bind(error_code.map(|c| c.as_str()))
    .bind(error_message)
    .bind(id.to_string())
    .execute(pool)
    .await?;

    Ok(())
}

#[derive(sqlx::FromRow)]
struct FileRow {
    id: String,
    original_name: String,
    current_name: String,
    original_path: String,
    current_path: String,
    extension: Option<String>,
    mime_type: Option<String>,
    size_bytes: Option<i64>,
    status: String,
    detected_at: String,
    ready_at: Option<String>,
    organized_at: Option<String>,
    last_seen_at: String,
    expires_at: Option<String>,
    group_id: Option<String>,
    source_context_id: Option<String>,
    error_code: Option<String>,
    error_message: Option<String>,
}

impl From<FileRow> for FileRecord {
    fn from(row: FileRow) -> Self {
        FileRecord {
            id: Uuid::parse_str(&row.id).unwrap_or_default(),
            original_name: row.original_name,
            current_name: row.current_name,
            original_path: row.original_path,
            current_path: row.current_path,
            extension: row.extension,
            mime_type: row.mime_type,
            size_bytes: row.size_bytes,
            status: FileStatus::parse(&row.status).unwrap_or(FileStatus::Error),
            detected_at: crate::parse_timestamp(&row.detected_at),
            ready_at: row.ready_at.as_deref().map(crate::parse_timestamp),
            organized_at: row.organized_at.as_deref().map(crate::parse_timestamp),
            last_seen_at: crate::parse_timestamp(&row.last_seen_at),
            expires_at: row.expires_at.as_deref().map(crate::parse_timestamp),
            group_id: row
                .group_id
                .as_deref()
                .and_then(|s| Uuid::parse_str(s).ok()),
            source_context_id: row
                .source_context_id
                .as_deref()
                .and_then(|s| Uuid::parse_str(s).ok()),
            error_code: row.error_code.as_deref().and_then(AppErrorCode::parse),
            error_message: row.error_message,
        }
    }
}
