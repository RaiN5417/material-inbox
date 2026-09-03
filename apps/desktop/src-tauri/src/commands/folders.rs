use std::path::PathBuf;
use std::sync::Mutex;

use file_watcher::WatcherHandle;
use storage::DbPool;
use tauri::State;

/// Settings-table key for the persisted watch list (a JSON array of path
/// strings) — same generic key/value table the locale and theme settings
/// already use.
const SETTINGS_KEY: &str = "watched_folders";

/// Reads the persisted watch list. Empty (not an error) if never set, so
/// callers that need a default can supply one themselves — see `app.rs`'s
/// startup, which falls back to the OS Downloads folder on first run.
pub async fn load_watched_folders(pool: &DbPool) -> Result<Vec<PathBuf>, String> {
    let raw = storage::get_setting(pool, SETTINGS_KEY)
        .await
        .map_err(|err| err.to_string())?;
    let Some(raw) = raw else {
        return Ok(Vec::new());
    };
    let paths: Vec<String> = serde_json::from_str(&raw).map_err(|err| err.to_string())?;
    Ok(paths.into_iter().map(PathBuf::from).collect())
}

pub async fn save_watched_folders(pool: &DbPool, folders: &[PathBuf]) -> Result<(), String> {
    let raw = serde_json::to_string(&to_strings(folders)).map_err(|err| err.to_string())?;
    storage::set_setting(pool, SETTINGS_KEY, &raw)
        .await
        .map_err(|err| err.to_string())
}

fn to_strings(folders: &[PathBuf]) -> Vec<String> {
    folders
        .iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect()
}

#[tauri::command]
pub async fn list_watched_folders(pool: State<'_, DbPool>) -> Result<Vec<String>, String> {
    Ok(to_strings(&load_watched_folders(&pool).await?))
}

/// Adds a folder to the live watch (no restart needed) and persists it.
#[tauri::command]
pub async fn add_watched_folder(
    pool: State<'_, DbPool>,
    watcher: State<'_, Mutex<WatcherHandle>>,
    path: String,
) -> Result<Vec<String>, String> {
    let candidate = PathBuf::from(&path);
    let mut folders = load_watched_folders(&pool).await?;
    if folders.contains(&candidate) {
        return Ok(to_strings(&folders));
    }

    watcher
        .lock()
        .map_err(|_| "watcher lock poisoned".to_string())?
        .add_path(&candidate)
        .map_err(|err| err.to_string())?;

    folders.push(candidate);
    save_watched_folders(&pool, &folders).await?;
    Ok(to_strings(&folders))
}

/// Stops watching a folder and drops it from the persisted list. The
/// underlying unwatch is best-effort — the persisted list is the source of
/// truth, so a folder that's already gone (or was never actually
/// subscribed) still comes out of it.
#[tauri::command]
pub async fn remove_watched_folder(
    pool: State<'_, DbPool>,
    watcher: State<'_, Mutex<WatcherHandle>>,
    path: String,
) -> Result<Vec<String>, String> {
    let candidate = PathBuf::from(&path);
    let mut folders = load_watched_folders(&pool).await?;
    folders.retain(|f| f != &candidate);

    let _ = watcher
        .lock()
        .map_err(|_| "watcher lock poisoned".to_string())?
        .remove_path(&candidate);

    save_watched_folders(&pool, &folders).await?;
    Ok(to_strings(&folders))
}
