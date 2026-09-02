-- Initial schema for Download Inbox.
-- See docs/data-model.md and docs/download_inbox_product_technical_spec_v0.2.md section 19.

CREATE TABLE groups (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    destination_path TEXT,
    icon TEXT,
    is_pinned INTEGER NOT NULL DEFAULT 0,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE files (
    id TEXT PRIMARY KEY,
    original_name TEXT NOT NULL,
    current_name TEXT NOT NULL,
    original_path TEXT NOT NULL,
    current_path TEXT NOT NULL,
    extension TEXT,
    mime_type TEXT,
    size_bytes INTEGER,
    status TEXT NOT NULL,
    detected_at TEXT NOT NULL,
    ready_at TEXT,
    organized_at TEXT,
    last_seen_at TEXT NOT NULL,
    expires_at TEXT,
    group_id TEXT,
    source_context_id TEXT,
    error_code TEXT,
    error_message TEXT,
    FOREIGN KEY (group_id) REFERENCES groups (id)
);

CREATE INDEX idx_files_status ON files (status);
CREATE INDEX idx_files_group_id ON files (group_id);

CREATE TABLE operations (
    id TEXT PRIMARY KEY,
    file_id TEXT NOT NULL,
    operation_type TEXT NOT NULL,
    source_path TEXT,
    destination_path TEXT,
    status TEXT NOT NULL,
    created_at TEXT NOT NULL,
    completed_at TEXT,
    undone_at TEXT,
    error_code TEXT,
    error_message TEXT,
    FOREIGN KEY (file_id) REFERENCES files (id)
);

CREATE INDEX idx_operations_file_id ON operations (file_id);
CREATE INDEX idx_operations_status ON operations (status);

CREATE TABLE batches (
    id TEXT PRIMARY KEY,
    created_at TEXT NOT NULL,
    closed_at TEXT,
    status TEXT NOT NULL
);

CREATE TABLE batch_files (
    batch_id TEXT NOT NULL,
    file_id TEXT NOT NULL,
    PRIMARY KEY (batch_id, file_id),
    FOREIGN KEY (batch_id) REFERENCES batches (id),
    FOREIGN KEY (file_id) REFERENCES files (id)
);

CREATE TABLE settings (
    key TEXT PRIMARY KEY,
    value_json TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
