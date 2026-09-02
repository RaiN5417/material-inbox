pub mod groups;
pub mod operations;
pub mod temporary;

use domain::FileStatus;
use storage::DbPool;
use tauri::State;
use uuid::Uuid;

/// Trivial IPC smoke test for the M0 skeleton. Real commands
/// (list_inbox, ...) land with their respective milestones — see spec
/// section 26.
#[tauri::command]
pub fn ping() -> &'static str {
    "pong"
}

/// Marks a file `Later` (spec section 8.1): the user defers the decision,
/// the file stays put, and it keeps showing in the main Inbox instead of
/// popping the Floating Card again.
#[tauri::command]
pub async fn mark_later(pool: State<'_, DbPool>, file_id: String) -> Result<(), String> {
    let id = Uuid::parse_str(&file_id).map_err(|err| err.to_string())?;
    storage::mark_status(&pool, id, FileStatus::Later, None, None)
        .await
        .map_err(|err| err.to_string())
}
