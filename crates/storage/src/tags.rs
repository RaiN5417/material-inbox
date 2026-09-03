use std::collections::HashMap;

use chrono::Utc;
use domain::Tag;
use uuid::Uuid;

use crate::{DbPool, StorageError};

/// All tags that exist, alphabetical — for an autocomplete/palette.
pub async fn list_tags(pool: &DbPool) -> Result<Vec<Tag>, StorageError> {
    let rows =
        sqlx::query_as::<_, TagRow>("SELECT id, name, created_at FROM tags ORDER BY name ASC")
            .fetch_all(pool)
            .await?;

    Ok(rows.into_iter().map(Into::into).collect())
}

/// Tags for every file that has any, in one query — the gallery needs this
/// for every visible card at once, not one row at a time.
pub async fn list_all_file_tags(pool: &DbPool) -> Result<HashMap<Uuid, Vec<Tag>>, StorageError> {
    let rows = sqlx::query_as::<_, FileTagRow>(
        "SELECT file_tags.file_id, tags.id, tags.name, tags.created_at
         FROM file_tags JOIN tags ON tags.id = file_tags.tag_id",
    )
    .fetch_all(pool)
    .await?;

    let mut map: HashMap<Uuid, Vec<Tag>> = HashMap::new();
    for row in rows {
        let Ok(file_id) = Uuid::parse_str(&row.file_id) else {
            continue;
        };
        map.entry(file_id).or_default().push(Tag {
            id: Uuid::parse_str(&row.id).unwrap_or_default(),
            name: row.name,
            created_at: crate::parse_timestamp(&row.created_at),
        });
    }
    Ok(map)
}

async fn get_or_create_tag(pool: &DbPool, tag_name: &str) -> Result<Tag, StorageError> {
    let existing =
        sqlx::query_as::<_, TagRow>("SELECT id, name, created_at FROM tags WHERE name = ?")
            .bind(tag_name)
            .fetch_optional(pool)
            .await?;

    Ok(match existing {
        Some(row) => row.into(),
        None => {
            let tag = Tag {
                id: Uuid::new_v4(),
                name: tag_name.to_string(),
                created_at: Utc::now(),
            };
            sqlx::query("INSERT INTO tags (id, name, created_at) VALUES (?, ?, ?)")
                .bind(tag.id.to_string())
                .bind(&tag.name)
                .bind(tag.created_at.to_rfc3339())
                .execute(pool)
                .await?;
            tag
        }
    })
}

/// Creates a tag with no file attached yet — the sidebar's "+" button, so a
/// tag can exist (and show up as a filter) before anything is tagged with
/// it. Same get-or-create semantics as `add_tag_to_file`.
pub async fn create_tag(pool: &DbPool, tag_name: &str) -> Result<Tag, StorageError> {
    get_or_create_tag(pool, tag_name).await
}

/// Tags `file_id` with `tag_name`, creating the tag first if it doesn't
/// exist yet. A no-op (not an error) if the file already has this tag.
pub async fn add_tag_to_file(
    pool: &DbPool,
    file_id: Uuid,
    tag_name: &str,
) -> Result<Tag, StorageError> {
    let tag = get_or_create_tag(pool, tag_name).await?;

    sqlx::query("INSERT OR IGNORE INTO file_tags (file_id, tag_id) VALUES (?, ?)")
        .bind(file_id.to_string())
        .bind(tag.id.to_string())
        .execute(pool)
        .await?;

    Ok(tag)
}

pub async fn remove_tag_from_file(
    pool: &DbPool,
    file_id: Uuid,
    tag_id: Uuid,
) -> Result<(), StorageError> {
    sqlx::query("DELETE FROM file_tags WHERE file_id = ? AND tag_id = ?")
        .bind(file_id.to_string())
        .bind(tag_id.to_string())
        .execute(pool)
        .await?;

    Ok(())
}

/// Deletes a tag entirely — `file_tags.tag_id` has `ON DELETE CASCADE`
/// (migration 0003), so every file's link to it goes with it; the files
/// themselves are untouched.
pub async fn delete_tag(pool: &DbPool, tag_id: Uuid) -> Result<(), StorageError> {
    sqlx::query("DELETE FROM tags WHERE id = ?")
        .bind(tag_id.to_string())
        .execute(pool)
        .await?;

    Ok(())
}

#[derive(sqlx::FromRow)]
struct TagRow {
    id: String,
    name: String,
    created_at: String,
}

impl From<TagRow> for Tag {
    fn from(row: TagRow) -> Self {
        Tag {
            id: Uuid::parse_str(&row.id).unwrap_or_default(),
            name: row.name,
            created_at: crate::parse_timestamp(&row.created_at),
        }
    }
}

#[derive(sqlx::FromRow)]
struct FileTagRow {
    file_id: String,
    id: String,
    name: String,
    created_at: String,
}
