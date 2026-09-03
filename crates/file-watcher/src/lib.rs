//! Normalizes raw OS filesystem events into `FsEvent`.
//!
//! MUST be event-driven (`notify` crate) — polling the Downloads folder in a
//! loop is forbidden, see spec section 15/21. Any Windows-specific API, if
//! ever needed, must stay behind this crate and not leak into `inbox-service`.

use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use notify::{RecommendedWatcher, RecursiveMode, Watcher as _};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FsEventKind {
    Create,
    Modify,
    Rename,
    Remove,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FsEvent {
    pub path: PathBuf,
    pub kind: FsEventKind,
    pub observed_at: DateTime<Utc>,
}

#[derive(Debug, thiserror::Error)]
pub enum WatcherError {
    #[error(transparent)]
    Notify(#[from] notify::Error),
}

/// Owns the OS-level watch subscription. MUST be kept alive for as long as
/// watching should continue — dropping it silently stops the watch (the same
/// lifetime trap as Tauri's `TrayIcon`; keep it in managed app state). A
/// single `notify` watcher can subscribe to multiple roots at once, so one
/// handle covers every user-configured watched folder rather than needing
/// one watcher instance per folder.
pub struct WatcherHandle(RecommendedWatcher);

impl WatcherHandle {
    /// Adds another folder to this watcher's live subscription — e.g. the
    /// Settings page's "add a watched folder", without restarting the app.
    pub fn add_path(&mut self, path: &Path) -> Result<(), WatcherError> {
        self.0.watch(path, RecursiveMode::NonRecursive)?;
        Ok(())
    }

    /// Stops watching a folder. Safe to call even if it was never watched
    /// (e.g. it had already been deleted out from under the watcher) —
    /// callers treat this as best-effort.
    pub fn remove_path(&mut self, path: &Path) -> Result<(), WatcherError> {
        self.0.unwatch(path)?;
        Ok(())
    }
}

/// Starts an event-driven, non-recursive watch on every path in `roots`
/// (spec section 15: no polling loops). Returns the handle to keep alive
/// alongside a channel of normalized events, tagged from whichever root
/// they came from, for the caller to consume.
pub fn watch(
    roots: &[PathBuf],
) -> Result<(WatcherHandle, mpsc::UnboundedReceiver<FsEvent>), WatcherError> {
    let (tx, rx) = mpsc::unbounded_channel();

    let mut watcher = notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
        match res {
            Ok(event) => {
                if let Some(kind) = normalize_kind(&event.kind) {
                    let observed_at = Utc::now();
                    for path in event.paths {
                        // Receiver may have been dropped (app shutting down); nothing to do.
                        let _ = tx.send(FsEvent {
                            path,
                            kind,
                            observed_at,
                        });
                    }
                }
            }
            Err(err) => tracing::warn!(?err, "file watcher error"),
        }
    })?;

    for root in roots {
        watcher.watch(root, RecursiveMode::NonRecursive)?;
    }

    Ok((WatcherHandle(watcher), rx))
}

fn normalize_kind(kind: &notify::EventKind) -> Option<FsEventKind> {
    use notify::event::ModifyKind;
    use notify::EventKind;

    match kind {
        EventKind::Create(_) => Some(FsEventKind::Create),
        EventKind::Modify(ModifyKind::Name(_)) => Some(FsEventKind::Rename),
        EventKind::Modify(_) => Some(FsEventKind::Modify),
        EventKind::Remove(_) => Some(FsEventKind::Remove),
        _ => None,
    }
}
