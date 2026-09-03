use chrono::{Duration as ChronoDuration, Utc};
use domain::{FileRecord, Operation, OperationStatus, OperationType};
use storage::DbPool;
use tauri::State;
use uuid::Uuid;

/// Spec section 25 MVP default.
const DEFAULT_TTL_DAYS: i64 = 7;

/// Marks a file `Temporary` with a TTL (spec section 25). Also used by the
/// Temporary panel's "Keep N more days" action — it's the same operation,
/// just re-applied from `CleanupReady` back to `Temporary`.
#[tauri::command]
pub async fn mark_temporary(
    pool: State<'_, DbPool>,
    file_id: String,
    ttl_days: Option<u32>,
) -> Result<FileRecord, String> {
    let file_id = Uuid::parse_str(&file_id).map_err(|err| err.to_string())?;
    let days = ttl_days.map(i64::from).unwrap_or(DEFAULT_TTL_DAYS);
    let expires_at = Utc::now() + ChronoDuration::days(days);

    storage::mark_temporary(&pool, file_id, expires_at)
        .await
        .map_err(|err| err.to_string())?;

    storage::get_file(&pool, file_id)
        .await
        .map_err(|err| err.to_string())?
        .ok_or_else(|| "file not found".to_string())
}

#[tauri::command]
pub async fn list_temporary(pool: State<'_, DbPool>) -> Result<Vec<FileRecord>, String> {
    storage::list_temporary(&pool)
        .await
        .map_err(|err| err.to_string())
}

/// Moves a file to the OS Recycle Bin (spec section 25) — terminal, no
/// in-app Undo; the user can still restore it from Windows' own Recycle Bin.
#[tauri::command]
pub async fn move_to_recycle_bin(pool: State<'_, DbPool>, file_id: String) -> Result<(), String> {
    let file_id = Uuid::parse_str(&file_id).map_err(|err| err.to_string())?;
    let file = storage::get_file(&pool, file_id)
        .await
        .map_err(|err| err.to_string())?
        .ok_or("file not found")?;
    let path = std::path::PathBuf::from(&file.current_path);

    let operation = Operation {
        id: Uuid::new_v4(),
        file_id,
        operation_type: OperationType::Trash,
        source_path: Some(file.current_path.clone()),
        destination_path: None,
        group_id: None,
        status: OperationStatus::Pending,
        created_at: Utc::now(),
        completed_at: None,
        undone_at: None,
        error_code: None,
        error_message: None,
    };
    storage::insert_operation(&pool, &operation)
        .await
        .map_err(|err| err.to_string())?;

    match file_operations::trash(&path).await {
        Ok(()) => {
            storage::mark_operation_completed(&pool, operation.id)
                .await
                .map_err(|err| err.to_string())?;
            storage::mark_trashed(&pool, file_id)
                .await
                .map_err(|err| err.to_string())
        }
        Err(err) => {
            let _ =
                storage::mark_operation_failed(&pool, operation.id, err.code, &err.message).await;
            Err(err.message)
        }
    }
}
