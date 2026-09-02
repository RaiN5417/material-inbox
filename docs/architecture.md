# Architecture

Full product/technical spec: [download_inbox_product_technical_spec_v0.2.md](download_inbox_product_technical_spec_v0.2.md).
This page is the short orientation map; the spec is the source of truth when
they disagree.

## Layers

```text
React UI (apps/desktop/src)
        │  Tauri IPC (typed commands, see src-tauri/src/commands)
        │  + events: file-ready, file-organized, floating-card:show(-batch), temporary-expired
        ▼
Tauri / Rust core (apps/desktop/src-tauri)
        │
        ├── inbox.rs           — orchestration: watcher → detector → event-engine → storage → UI
        ├── floating_card.rs   — the Floating Card window (single + batch)
        ├── temporary.rs       — background sweep: expired Temporary → CleanupReady
        │
        ├── file-watcher       — OS events → normalized FsEvent (event-driven, no polling)
        ├── download-detector  — is this path a finished, operable file yet?
        ├── event-engine       — batches ready files close together into one UI notification
        ├── file-operations    — the only crate allowed to move/rename/trash files
        ├── storage            — SQLite pool + migrations + repositories
        └── domain              — pure model shared by everything above (no Tauri/SQLite/Windows deps)
```

`domain` has no dependents outside the workspace's own crates — it must stay
compilable without Tauri, SQLite, or any Windows API in scope.

`inbox.rs`, `floating_card.rs`, and `temporary.rs` are the app's
`inbox-service` orchestration (spec section 18) living directly in the Tauri
crate rather than as their own workspace member — their job is inherently
Tauri-coupled (notifying the UI means emitting a Tauri event), and pulling
them out only pays off once something other than this app needs to reuse
them.

## Request/event flow (steady state)

```text
file lands in Downloads
  → file-watcher normalizes the raw OS event
  → download-detector runs the stability check (ignores .crdownload/.part/.tmp)
  → storage records the file (Detected → Pending)
  → event-engine holds it open for up to 2s in case more files are arriving
  → floating_card shows one card (single file or the whole batch)
  → user clicks Group / Temporary / Later
  → commands/{groups,temporary}.rs run preflight → operation log → file-operations → verify → commit
  → UI updates via file-organized / file-ready events
```

Undo reverses a completed move via the same operation log
(`commands/operations.rs`), and `temporary.rs`'s sweep runs independently of
user action to promote expired files.

## Build order (for reference — all landed)

Crates and features landed in this order (spec section 62); still the right
order to extend in:

1. domain model
2. SQLite + migrations
3. watcher
4. completion detector
5. event engine
6. inbox-service (orchestration)
7. Tauri commands
8. Floating Card UI
9. file-operations (move/rename)
10. undo
11. batch
12. temporary lifecycle
13. browser extension (v0.2 — not started)

## Current milestone

M0–M6 (full MVP feature set per spec section 49) are done. M7 (README, docs,
benchmarks, first release — this pass) is in progress. Section 64 has the
full MVP acceptance checklist; anything unchecked there is what's left.
