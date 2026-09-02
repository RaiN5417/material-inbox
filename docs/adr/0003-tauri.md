# ADR-003: Tauri 2 for the desktop shell

**Decision:** Tauri 2 hosting a React + TypeScript frontend over a Rust core.

**Reasons:** keeps the resident process in Rust while getting web frontend
iteration speed for the UI, plus built-in system tray, native windows, and
packaging — all lighter-weight than Electron, which matches the project's
lightweight-by-design goal.
