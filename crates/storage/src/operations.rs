use domain::{AppErrorCode, Operation, OperationStatus, OperationType};
use uuid::Uuid;

use crate::{DbPool, StorageError};

const OPERATION_COLUMNS: &str = "id, file_id, operation_type, source_path, destination_path, \
     group_id, status, created_at, completed_at, undone_at, error_code, error_message";

pub async fn get_operation(pool: &DbPool, id: Uuid) -> Result<Option<Operation>, StorageError> {
    let row = sqlx::query_as::<_, OperationRow>(&format!(
        "SELECT {OPERATION_COLUMNS} FROM operations WHERE id = ?"
    ))
    .bind(id.to_string())
    .fetch_optional(pool)
    .await?;

    Ok(row.map(Into::into))
}

/// Most recent operations for the History view (spec section 11), newest
/// first.
pub async fn list_operations(pool: &DbPool, limit: i64) -> Result<Vec<Operation>, StorageError> {
    let rows = sqlx::query_as::<_, OperationRow>(&format!(
        "SELECT {OPERATION_COLUMNS} FROM operations ORDER BY created_at DESC LIMIT ?"
    ))
    .bind(limit)
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(Into::into).collect())
}

/// Operations still `Pending` — normally a fleeting in-memory state, so any
/// row still in it at startup means the app crashed between logging the
/// operation and committing its outcome (spec section 39).
pub async fn list_pending_operations(pool: &DbPool) -> Result<Vec<Operation>, StorageError> {
    let rows = sqlx::query_as::<_, OperationRow>(&format!(
        "SELECT {OPERATION_COLUMNS} FROM operations WHERE status = ?"
    ))
    .bind(OperationStatus::Pending.as_str())
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(Into::into).collect())
}

/// Writes the operation as `Pending` *before* the filesystem action executes
/// (spec section 18/39) — if the app crashes mid-move, this row is what
/// startup reconciliation would use to figure out what happened.
pub async fn insert_operation(pool: &DbPool, op: &Operation) -> Result<(), StorageError> {
    sqlx::query(
        "INSERT INTO operations (
            id, file_id, operation_type, source_path, destination_path, group_id,
            status, created_at, completed_at, undone_at, error_code, error_message
        ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(op.id.to_string())
    .bind(op.file_id.to_string())
    .bind(op.operation_type.as_str())
    .bind(&op.source_path)
    .bind(&op.destination_path)
    .bind(op.group_id.map(|id| id.to_string()))
    .bind(op.status.as_str())
    .bind(op.created_at.to_rfc3339())
    .bind(op.completed_at.map(|t| t.to_rfc3339()))
    .bind(op.undone_at.map(|t| t.to_rfc3339()))
    .bind(op.error_code.map(|c| c.as_str()))
    .bind(&op.error_message)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn mark_operation_completed(pool: &DbPool, id: Uuid) -> Result<(), StorageError> {
    sqlx::query("UPDATE operations SET status = ?, completed_at = ? WHERE id = ?")
        .bind(OperationStatus::Completed.as_str())
        .bind(chrono::Utc::now().to_rfc3339())
        .bind(id.to_string())
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn mark_operation_failed(
    pool: &DbPool,
    id: Uuid,
    error_code: AppErrorCode,
    error_message: &str,
) -> Result<(), StorageError> {
    sqlx::query("UPDATE operations SET status = ?, error_code = ?, error_message = ? WHERE id = ?")
        .bind(OperationStatus::Failed.as_str())
        .bind(error_code.as_str())
        .bind(error_message)
        .bind(id.to_string())
        .execute(pool)
        .await?;

    Ok(())
}

/// Undo doesn't get its own log row — it just stamps `undone_at` on the
/// original operation (spec section 19.3's `operations.undone_at` column is
/// on the same row, not a separate one).
pub async fn mark_operation_undone(pool: &DbPool, id: Uuid) -> Result<(), StorageError> {
    sqlx::query("UPDATE operations SET status = ?, undone_at = ? WHERE id = ?")
        .bind(OperationStatus::Undone.as_str())
        .bind(chrono::Utc::now().to_rfc3339())
        .bind(id.to_string())
        .execute(pool)
        .await?;

    Ok(())
}

#[derive(sqlx::FromRow)]
struct OperationRow {
    id: String,
    file_id: String,
    operation_type: String,
    source_path: Option<String>,
    destination_path: Option<String>,
    group_id: Option<String>,
    status: String,
    created_at: String,
    completed_at: Option<String>,
    undone_at: Option<String>,
    error_code: Option<String>,
    error_message: Option<String>,
}

impl From<OperationRow> for Operation {
    fn from(row: OperationRow) -> Self {
        Operation {
            id: Uuid::parse_str(&row.id).unwrap_or_default(),
            file_id: Uuid::parse_str(&row.file_id).unwrap_or_default(),
            operation_type: OperationType::parse(&row.operation_type)
                .unwrap_or(OperationType::Move),
            source_path: row.source_path,
            destination_path: row.destination_path,
            group_id: row
                .group_id
                .as_deref()
                .and_then(|s| Uuid::parse_str(s).ok()),
            status: OperationStatus::parse(&row.status).unwrap_or(OperationStatus::Failed),
            created_at: crate::parse_timestamp(&row.created_at),
            completed_at: row.completed_at.as_deref().map(crate::parse_timestamp),
            undone_at: row.undone_at.as_deref().map(crate::parse_timestamp),
            error_code: row.error_code.as_deref().and_then(AppErrorCode::parse),
            error_message: row.error_message,
        }
    }
}
