# ADR-006: Operation Log as the basis for Undo

**Decision:** every file mutation (move/rename/restore/trash) is recorded as
an `Operation` row before it executes. Undo replays from this log, never
from UI-side history.

**Reason:** a file management tool's first responsibility is not losing or
misplacing files. An operation log gives a durable, crash-recoverable record
that both Undo and startup reconciliation (spec section 39) can rely on —
UI state alone can't survive a crash or restart.
