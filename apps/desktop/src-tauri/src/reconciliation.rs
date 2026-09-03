//! Startup crash recovery (spec section 39).
//!
//! Every mutating operation is logged as `Pending` *before* it executes, then
//! flipped to `Completed`/`Failed` after — see spec section 18/35. If the app
//! crashes in between, that row is stuck at `Pending` forever unless
//! something checks for it at the next startup. This module is that check:
//! it looks at what's actually on disk to figure out what happened, rather
//! than guessing from the DB alone.

use domain::{AppErrorCode, FileStatus, Operation, OperationType};
use storage::DbPool;

pub async fn run(pool: &DbPool) {
    let pending = match storage::list_pending_operations(pool).await {
        Ok(ops) => ops,
        Err(err) => {
            tracing::error!(?err, "failed to list pending operations for reconciliation");
            return;
        }
    };

    if pending.is_empty() {
        return;
    }
    tracing::warn!(
        count = pending.len(),
        "reconciling operations left pending by a previous crash"
    );

    for op in pending {
        reconcile_one(pool, op).await;
    }
}

async fn reconcile_one(pool: &DbPool, op: Operation) {
    match op.operation_type {
        OperationType::Move => reconcile_move(pool, op).await,
        // Nothing produces a pending Trash/Restore/Rename yet beyond Move,
        // but if one shows up, flag it rather than guess what to do with it.
        _ => {
            let _ = storage::mark_operation_failed(
                pool,
                op.id,
                AppErrorCode::InvalidPath,
                "operation left pending by a crash; this type isn't covered by reconciliation",
            )
            .await;
        }
    }
}

async fn reconcile_move(pool: &DbPool, op: Operation) {
    let (Some(source), Some(destination)) =
        (op.source_path.as_deref(), op.destination_path.as_deref())
    else {
        let _ = storage::mark_operation_failed(
            pool,
            op.id,
            AppErrorCode::InvalidPath,
            "move operation is missing its source/destination path",
        )
        .await;
        return;
    };

    let source_exists = tokio::fs::metadata(source).await.is_ok();
    let dest_exists = tokio::fs::metadata(destination).await.is_ok();

    match (source_exists, dest_exists) {
        // Spec section 39's worked example: source gone, destination there
        // → the move almost certainly finished right before the crash.
        (false, true) => {
            tracing::info!(operation_id = %op.id, "reconciled: move appears to have completed before the crash");
            let _ = storage::mark_operation_completed(pool, op.id).await;

            let file_name = std::path::Path::new(destination)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default()
                .to_string();

            match op.group_id {
                Some(group_id) => {
                    let _ =
                        storage::assign_group(pool, op.file_id, group_id, &file_name, destination)
                            .await;
                }
                // No group on record — land it back in the Inbox rather
                // than guess a status for it.
                None => {
                    let _ = storage::mark_status(pool, op.file_id, FileStatus::Pending, None, None)
                        .await;
                }
            }
        }
        // Source untouched, destination never appeared: the move never ran.
        // Nothing to fix on the file — it's exactly where it always was.
        (true, false) => {
            tracing::info!(operation_id = %op.id, "reconciled: move never executed before the crash");
            let _ = storage::mark_operation_failed(
                pool,
                op.id,
                AppErrorCode::InvalidPath,
                "interrupted by a crash before the move ran",
            )
            .await;
        }
        // Neither path exists — genuinely unaccounted for.
        (false, false) => {
            tracing::warn!(operation_id = %op.id, "reconciled: neither source nor destination exist after a crash");
            let _ = storage::mark_operation_failed(
                pool,
                op.id,
                AppErrorCode::SourceMissing,
                "file missing after a crash; neither its old nor new location exists",
            )
            .await;
            let _ = storage::mark_status(pool, op.file_id, FileStatus::Missing, None, None).await;
        }
        // Both exist: likely a cross-volume copy that succeeded before the
        // crash hit, right before source cleanup ran. Don't guess-delete the
        // source (safe-by-default) — flag it for a human instead.
        (true, true) => {
            tracing::warn!(operation_id = %op.id, "reconciled: both source and destination exist after a crash, needs manual cleanup");
            let _ = storage::mark_operation_failed(
                pool,
                op.id,
                AppErrorCode::CrossVolumeMoveFailed,
                "both the old and new locations exist after a crash — check manually before deleting either",
            )
            .await;
        }
    }
}
