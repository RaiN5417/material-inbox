# Performance

Real measurements only, per the rules below. Targets (not promises) live in
spec section 32.

## Test environment

- OS: Windows 10 Pro, build 10.0.19044
- CPU: Intel Core i5-8400 @ 2.80GHz (6 cores, no HT)
- RAM: 16 GB
- Rust: 1.98.0
- Build: `cargo build --release` (optimized, no debug assertions)
- Binary size: 14.6 MB (`download-inbox.exe`, unstripped)
- Date: 2026-09-02

## Idle

Measured via `Get-Process` over a 10s window (`(cpu_after - cpu_before) /
wall_seconds` for CPU; `WorkingSet64` for RAM).

| Scenario | CPU (of 1 core) | Working set |
|---|---|---|
| Main window open | 0.16% | 34.3 MB |
| Hidden to tray (`WM_CLOSE` → app.rs's hide handler) | 0% | 34.3 MB |

Both comfortably clear the MVP target (< 35 MB) though not yet the stretch
goal (< 20 MB) — most of this is the WebView2 host process overhead that
comes with any Tauri app, not something specific to this app's own code.

## Event load: 100-file burst

Method: wrote 100 tiny (~20 byte) `.txt` files into the watched Downloads
folder as fast as `Set-Content` in a loop allows, then polled the SQLite
`files` table until all 100 rows (matched by a unique filename prefix,
`detected_at` after the write started) left the `detected`/`waiting_stable`
states.

- **100 / 100 files accounted for in 1.28s**, zero dropped events.
- Process stayed `Responding = True` throughout — the UI thread never
  blocked (each candidate file runs its own stability-check task, per spec
  section 20.2).
- Working set grew from 32.6 MB → 34.8 MB over the burst (+2.2 MB for 100
  in-flight tracking tasks + DB rows).

Caveat: these are trivially small files, so the stability check settles in
close to its minimum time (2 × 300ms). This measures event-handling
throughput and "nothing gets dropped/blocked," not large-file I/O
performance — a burst of many large files would take longer per-file to
reach `Ready` (bounded by disk write speed, not by anything in this app),
but wouldn't change the "no dropped events" result since each file is
tracked independently.

Batch-engine correctness (client observed once directly, source of the "1
card, not N" requirement) is also covered by `crates/event-engine`'s unit
tests, which assert the exact merge/split behavior deterministically
without relying on wall-clock timing in CI.

## Not yet measured

- 1,000 synthetic file burst (spec section 32's larger target).
- SQLite write latency under sustained load.
- Startup time (cold vs. warm).

Add rows here as they're measured — never estimate or round up.
