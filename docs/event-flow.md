# Event flow

Full detail: spec sections 20–22. Three separate concerns — don't collapse
them into one step, they solve different problems:

```text
raw OS fs events
      │
      ▼
Dedup in-flight paths            — solves "one path, many raw events": a HashSet
      │  key = normalized_path      of paths already being tracked stops a burst of
      │                             Modify events from spawning a second stability-check
      │                             task for the same file (see inbox.rs's `in_flight`)
      ▼
Stability check (300ms × 2)     — solves "is this file actually done downloading?"
      │  size/mtime unchanged for 2 rounds, shareable read succeeds → Ready
      │  .crdownload / .part / .tmp / .download are filtered but their rename
      │  to a final name is itself a signal, not noise
      ▼
Batch window (2s, max 5s/100 items) — solves UX: don't pop 20 cards for 20 files
      │  (crates/event-engine)
      ▼
Floating Card / Inbox
```

There's no separate timer-based debounce stage on the raw filesystem
events — the stability check already absorbs that noise (repeated Modify
events just reset its counter), so a dedicated debounce layer would have
been solving the same problem twice.

Timeouts: stability max wait 120s → `PendingRetry` (not `Error`) with
low-frequency background retry (every 10s). Only persistent failure (e.g.
permissions) becomes `Error`.
