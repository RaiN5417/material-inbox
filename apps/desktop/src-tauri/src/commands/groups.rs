use chrono::Utc;
use domain::{FileRecord, Group, Operation, OperationStatus, OperationType};
use storage::DbPool;
use tauri::{AppHandle, Emitter, State};
use uuid::Uuid;

/// Emitted after a successful move so the UI can offer Undo without a
/// dedicated "list operations" round trip — see spec section 26's event list.
const EVENT_FILE_ORGANIZED: &str = "file-organized";

#[derive(serde::Serialize, Clone)]
struct OrganizedEvent {
    file: FileRecord,
    operation_id: Uuid,
}

#[tauri::command]
pub async fn create_group(
    pool: State<'_, DbPool>,
    name: String,
    destination_path: String,
) -> Result<Group, String> {
    let now = Utc::now();
    let group = Group {
        id: Uuid::new_v4(),
        name,
        destination_path: Some(destination_path),
        icon: None,
        is_pinned: false,
        sort_order: 0,
        created_at: now,
        updated_at: now,
    };

    storage::insert_group(&pool, &group)
        .await
        .map_err(|err| err.to_string())?;
    Ok(group)
}

#[tauri::command]
pub async fn list_groups(pool: State<'_, DbPool>) -> Result<Vec<Group>, String> {
    storage::list_groups(&pool)
        .await
        .map_err(|err| err.to_string())
}

/// Moves one file into a group's destination folder (spec section 23).
///
/// Takes a single file rather than the `Vec<file_id>` shape spec section 26
/// eventually wants — the only caller today is the Floating Card acting on
/// the one file it's showing. A multi-select Inbox view can loop this per
/// id when it lands; that's a UI-layer concern, not a reason to build batch
/// plumbing here now.
#[tauri::command]
pub async fn assign_group(
    app: AppHandle,
    pool: State<'_, DbPool>,
    file_id: String,
    group_id: String,
) -> Result<FileRecord, String> {
    let file_id = Uuid::parse_str(&file_id).map_err(|err| err.to_string())?;
    let group_id = Uuid::parse_str(&group_id).map_err(|err| err.to_string())?;

    let file = storage::get_file(&pool, file_id)
        .await
        .map_err(|err| err.to_string())?
        .ok_or("file not found")?;
    let group = storage::get_group(&pool, group_id)
        .await
        .map_err(|err| err.to_string())?
        .ok_or("group not found")?;
    let destination_dir = group
        .destination_path
        .ok_or("group has no destination path set")?;
    let destination_dir = std::path::PathBuf::from(destination_dir);

    tokio::fs::create_dir_all(&destination_dir)
        .await
        .map_err(|err| err.to_string())?;

    let source = std::path::PathBuf::from(&file.current_path);
    let destination =
        file_operations::resolve_destination(&destination_dir, &file.current_name).await;

    // Operation log is written *before* the filesystem action executes
    // (spec section 18/39), so a crash mid-move leaves a reconcilable trail.
    let operation = Operation {
        id: Uuid::new_v4(),
        file_id,
        operation_type: OperationType::Move,
        source_path: Some(source.to_string_lossy().into_owned()),
        destination_path: Some(destination.to_string_lossy().into_owned()),
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

    match file_operations::execute_move(&source, &destination).await {
        Ok(_size) => {
            storage::mark_operation_completed(&pool, operation.id)
                .await
                .map_err(|err| err.to_string())?;

            let final_name = destination
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(&file.current_name)
                .to_string();
            let destination_str = destination.to_string_lossy().into_owned();

            storage::assign_group(&pool, file_id, group_id, &final_name, &destination_str)
                .await
                .map_err(|err| err.to_string())?;

            let updated = storage::get_file(&pool, file_id)
                .await
                .map_err(|err| err.to_string())?
                .ok_or_else(|| "file disappeared after being organized".to_string())?;

            let _ = app.emit(
                EVENT_FILE_ORGANIZED,
                OrganizedEvent {
                    file: updated.clone(),
                    operation_id: operation.id,
                },
            );

            Ok(updated)
        }
        Err(err) => {
            let _ =
                storage::mark_operation_failed(&pool, operation.id, err.code, &err.message).await;
            let _ = storage::mark_status(
                &pool,
                file_id,
                domain::FileStatus::Error,
                Some(err.code),
                Some(&err.message),
            )
            .await;
            Err(err.message)
        }
    }
}
