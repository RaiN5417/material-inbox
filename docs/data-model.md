# Data model

SQLite, single file, WAL mode, foreign keys on (see [migrations/0001_init.sql](../migrations/0001_init.sql)).

## Tables

| Table | Purpose |
|---|---|
| `files` | One row per tracked file, `status` drives the lifecycle state machine (spec section 13). |
| `groups` | User-defined destinations ("projects"). MVP: one primary group per file. |
| `operations` | Audit + undo log for every move/rename/restore/trash. Undo replays this, never UI history. |
| `batches` / `batch_files` | Schema reserved for persisted batch history; unused so far — `event-engine`'s batching is in-memory only (a closed batch becomes one Floating Card and is never written to SQLite). Wire this up if/when batch history needs to survive a restart. |
| `settings` | Key/value app settings (`value_json` blob). Unused so far — no Settings UI exists yet. |

`files.status` is the `FileStatus` enum in [`crates/domain`](../crates/domain/src/file.rs) —
kept in sync by hand since sqlx doesn't derive SQL-side enum constraints for SQLite.

## Why no ORM

Direct `sqlx` queries against typed `domain` structs. The schema is small
enough (6 tables) that an ORM would add indirection without buying much;
see spec section 48 on dependency control.
