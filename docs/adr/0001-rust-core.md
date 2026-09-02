# ADR-001: Rust for the resident core

**Decision:** the always-running background core (watcher, event engine,
storage, file operations) is written in Rust.

**Reasons:** low idle background memory, direct OS integration, single
binary distribution, and it's a stronger engineering showcase than the
alternatives.

**Rejected:** a resident Python process, a Node backend, or an Electron main
process — all heavier at idle and a weaker fit for a "must stay near-zero
CPU/RAM when idle" product.
