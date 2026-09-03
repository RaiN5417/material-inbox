use storage::DbPool;
use tauri::State;

/// `value` is a raw JSON string — callers encode/decode their own shape.
/// Currently just holds the UI locale; a natural home for the rest of
/// Settings when that view gets built out.
#[tauri::command]
pub async fn get_setting(pool: State<'_, DbPool>, key: String) -> Result<Option<String>, String> {
    storage::get_setting(&pool, &key)
        .await
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn set_setting(
    pool: State<'_, DbPool>,
    key: String,
    value: String,
) -> Result<(), String> {
    storage::set_setting(&pool, &key, &value)
        .await
        .map_err(|err| err.to_string())
}
