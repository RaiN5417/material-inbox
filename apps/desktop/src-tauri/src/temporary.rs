//! Periodically promotes expired `Temporary` files to `CleanupReady` (spec
//! section 13/25) so the Temporary panel picks them up without requiring the
//! app to be actively watching that exact moment.

use std::time::Duration;

use storage::DbPool;
use tauri::{AppHandle, Emitter};

const SWEEP_INTERVAL: Duration = Duration::from_secs(60);

/// Event fired when one or more files just became `CleanupReady`, so an
/// open Temporary panel can refresh without polling.
const EVENT_EXPIRED: &str = "temporary-expired";

pub async fn run(app: AppHandle, pool: DbPool) {
    loop {
        match storage::sweep_expired(&pool).await {
            Ok(expired) if !expired.is_empty() => {
                let _ = app.emit(EVENT_EXPIRED, &expired);
            }
            Ok(_) => {}
            Err(err) => tracing::error!(?err, "failed to sweep expired temporary files"),
        }
        tokio::time::sleep(SWEEP_INTERVAL).await;
    }
}
