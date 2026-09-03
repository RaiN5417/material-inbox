use domain::{FileRecord, Operation};
use storage::DbPool;
use tauri::State;

/// Loads the Inbox from the DB (spec section 11: pending / later / failed
/// files) so it survives a restart instead of only reflecting whatever this
/// session happened to see live.
#[tauri::command]
pub async fn list_inbox(pool: State<'_, DbPool>) -> Result<Vec<FileRecord>, String> {
    storage::list_inbox(&pool)
        .await
        .map_err(|err| err.to_string())
}

/// Most recent operations for the History view (spec section 11). A
/// completed `Move` that hasn't been undone yet is undoable from here via
/// the same `undo_operation` command the Floating Card's live session uses
/// — History is what makes Undo durable across a restart, not a special
/// path of its own.
#[tauri::command]
pub async fn list_operations(pool: State<'_, DbPool>) -> Result<Vec<Operation>, String> {
    storage::list_operations(&pool, 200)
        .await
        .map_err(|err| err.to_string())
}
