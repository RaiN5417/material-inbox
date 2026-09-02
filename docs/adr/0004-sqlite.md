# ADR-004: SQLite for storage

**Decision:** SQLite via `sqlx`, one file in the per-user app data directory.

**Reasons:** local-first with no separate service to run, single-user
scale is trivially within SQLite's range, portable, and gives real
transaction/history semantics for the operation log — `sqlx` fits the
Tokio-based async core cleanly.

**Constraint:** don't add `rusqlite` alongside `sqlx` — that would duplicate
the SQLite binding and double the dependency surface for no benefit.
