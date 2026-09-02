use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A user-defined context ("project") a file can be filed into.
/// MVP keeps this single-parent (one file → one primary group).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    pub id: Uuid,
    pub name: String,
    pub destination_path: Option<String>,
    pub icon: Option<String>,
    pub is_pinned: bool,
    pub sort_order: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
