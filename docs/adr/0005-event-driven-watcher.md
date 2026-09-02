# ADR-005: Event-driven watcher, no polling

**Decision:** file system monitoring uses OS-native change notifications
(the `notify` crate), never a `loop { scan Downloads }`.

**Reasons:** polling burns CPU/disk/battery for no latency benefit and
doesn't fit a product whose idle-resource target is near zero. Event-driven
watching is also simply the more correct tool for the job.

**Constraint:** if Windows-specific APIs (`ReadDirectoryChangesW`) are ever
needed for an edge case `notify` can't handle well, they must stay wrapped
inside `crates/file-watcher` and never leak into `inbox-service` or the UI.
