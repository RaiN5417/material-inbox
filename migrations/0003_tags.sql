-- Lightweight many-to-many tags, independent of groups (spec section 12:
-- "未来可增加 Tags,多对多" — groups stay single-parent via a physical move,
-- tags are a pure DB-side label with no filesystem effect).

CREATE TABLE tags (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    created_at TEXT NOT NULL
);

CREATE TABLE file_tags (
    file_id TEXT NOT NULL,
    tag_id TEXT NOT NULL,
    PRIMARY KEY (file_id, tag_id),
    FOREIGN KEY (file_id) REFERENCES files (id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES tags (id) ON DELETE CASCADE
);

CREATE INDEX idx_file_tags_tag_id ON file_tags (tag_id);
