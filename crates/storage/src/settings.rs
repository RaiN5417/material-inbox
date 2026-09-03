use chrono::Utc;

use crate::{DbPool, StorageError};

/// `value` is a raw JSON-encoded string (matches the `value_json` column
/// name) — the caller decides the shape; storage just persists it. Used for
/// UI locale today, a natural home for the rest of Settings later.
pub async fn get_setting(pool: &DbPool, key: &str) -> Result<Option<String>, StorageError> {
    let row: Option<(String,)> = sqlx::query_as("SELECT value_json FROM settings WHERE key = ?")
        .bind(key)
        .fetch_optional(pool)
        .await?;

    Ok(row.map(|(v,)| v))
}

pub async fn set_setting(pool: &DbPool, key: &str, value_json: &str) -> Result<(), StorageError> {
    sqlx::query(
        "INSERT INTO settings (key, value_json, updated_at) VALUES (?, ?, ?)
         ON CONFLICT(key) DO UPDATE SET value_json = excluded.value_json, updated_at = excluded.updated_at",
    )
    .bind(key)
    .bind(value_json)
    .bind(Utc::now().to_rfc3339())
    .execute(pool)
    .await?;

    Ok(())
}
