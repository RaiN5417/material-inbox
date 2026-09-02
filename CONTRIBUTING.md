# Contributing

Thanks for looking at Download Inbox. This is an open-source portfolio
project with a fairly opinionated spec — read
[docs/download_inbox_product_technical_spec_v0.2.md](docs/download_inbox_product_technical_spec_v0.2.md)
section 0 before proposing anything non-trivial; it lists the MUST/MUST NOT
constraints (no AI, no cloud, no telemetry, Windows-first, event-driven
watcher, etc.) and they're not up for debate per-PR.

## Setup

See the root [README](README.md#getting-started).

## Before opening a PR

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `pnpm lint && pnpm typecheck && pnpm build` (from `apps/desktop`)

All four run in CI; a red CI run won't get reviewed until it's green.

## Code style

- Rust: no `unwrap()` on a path a user can trigger; errors propagate as
  `AppErrorCode` (see `crates/domain/src/error.rs`), never raw OS text
  reaching the UI. `domain` stays free of Tauri/SQLite/Windows deps.
- TypeScript: `strict: true`, no `any` as an escape hatch, components never
  touch the filesystem directly — only through the Tauri command layer.
- New dependency? Answer spec section 48's five questions in the PR
  description first.

## Where to start

`good first issue` label, or see spec section 67 for the recommended first
issue list.
