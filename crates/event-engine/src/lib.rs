//! Merges files that became ready close together into one batch instead of
//! notifying the UI once per file (spec section 9/22) — otherwise a 20-file
//! download would pop 20 Floating Cards.
//!
//! Operates on `ReadyFileEvent`s (post completion-check), not raw filesystem
//! events — that debouncing/noise-filtering already happened upstream in
//! `file-watcher` + `download-detector`.

use std::time::{Duration, Instant};

use domain::FileRecord;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

#[derive(Debug, Clone, Copy)]
pub struct BatchConfig {
    /// How long to wait for another file before closing the batch — reset
    /// each time a new file arrives (spec section 9.1 default: 2s).
    pub window: Duration,
    /// Hard cap on how long one batch can stay open, so a continuous stream
    /// of downloads can't keep it open forever (spec section 22).
    pub max_age: Duration,
    pub max_items: usize,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            window: Duration::from_millis(2000),
            max_age: Duration::from_secs(5),
            max_items: 100,
        }
    }
}

/// Spawns the batch aggregator on the current Tokio runtime. Feed ready
/// files into the returned sender; closed batches (always at least one
/// file) arrive on the returned receiver.
pub fn spawn(
    config: BatchConfig,
) -> (
    UnboundedSender<FileRecord>,
    UnboundedReceiver<Vec<FileRecord>>,
) {
    let (in_tx, mut in_rx) = mpsc::unbounded_channel::<FileRecord>();
    let (out_tx, out_rx) = mpsc::unbounded_channel::<Vec<FileRecord>>();

    tokio::spawn(async move {
        loop {
            let Some(first) = in_rx.recv().await else {
                break;
            };
            let batch_start = Instant::now();
            let mut batch = vec![first];

            loop {
                if batch.len() >= config.max_items {
                    break;
                }
                let elapsed = batch_start.elapsed();
                if elapsed >= config.max_age {
                    break;
                }
                let wait = config.window.min(config.max_age - elapsed);

                match tokio::time::timeout(wait, in_rx.recv()).await {
                    Ok(Some(file)) => batch.push(file),
                    Ok(None) => {
                        let _ = out_tx.send(batch);
                        return;
                    }
                    Err(_elapsed) => break,
                }
            }

            if out_tx.send(batch).is_err() {
                break;
            }
        }
    });

    (in_tx, out_rx)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use domain::FileStatus;
    use uuid::Uuid;

    fn dummy_file(name: &str) -> FileRecord {
        let now = Utc::now();
        FileRecord {
            id: Uuid::new_v4(),
            original_name: name.to_string(),
            current_name: name.to_string(),
            original_path: name.to_string(),
            current_path: name.to_string(),
            extension: None,
            mime_type: None,
            size_bytes: Some(1),
            status: FileStatus::Pending,
            detected_at: now,
            ready_at: Some(now),
            organized_at: None,
            last_seen_at: now,
            expires_at: None,
            group_id: None,
            source_context_id: None,
            error_code: None,
            error_message: None,
        }
    }

    #[tokio::test]
    async fn files_arriving_within_the_window_share_one_batch() {
        let (tx, mut rx) = spawn(BatchConfig {
            window: Duration::from_millis(60),
            max_age: Duration::from_secs(5),
            max_items: 100,
        });

        tx.send(dummy_file("a.png")).unwrap();
        tokio::time::sleep(Duration::from_millis(10)).await;
        tx.send(dummy_file("b.png")).unwrap();

        let batch = rx.recv().await.unwrap();
        assert_eq!(batch.len(), 2);
    }

    #[tokio::test]
    async fn files_arriving_apart_get_separate_batches() {
        let (tx, mut rx) = spawn(BatchConfig {
            window: Duration::from_millis(30),
            max_age: Duration::from_secs(5),
            max_items: 100,
        });

        tx.send(dummy_file("a.png")).unwrap();
        let first = rx.recv().await.unwrap();

        tokio::time::sleep(Duration::from_millis(60)).await;
        tx.send(dummy_file("b.png")).unwrap();
        let second = rx.recv().await.unwrap();

        assert_eq!(first.len(), 1);
        assert_eq!(second.len(), 1);
    }

    #[tokio::test]
    async fn batch_closes_at_max_items() {
        let (tx, mut rx) = spawn(BatchConfig {
            window: Duration::from_millis(500),
            max_age: Duration::from_secs(5),
            max_items: 3,
        });

        for i in 0..3 {
            tx.send(dummy_file(&format!("f{i}.png"))).unwrap();
        }

        let batch = rx.recv().await.unwrap();
        assert_eq!(batch.len(), 3);
    }
}
