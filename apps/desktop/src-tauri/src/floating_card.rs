//! The Floating Card: a non-focus-stealing popup shown at the bottom-right
//! of whichever monitor the mouse is on when one or more files become ready
//! (spec section 9/10). Created once at startup and reused rather than
//! spawned per download — that would be wasteful and flicker-prone.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use domain::FileRecord;
use serde::Serialize;
use tauri::{
    AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, Position, Size, WebviewUrl,
    WebviewWindowBuilder,
};

const WINDOW_LABEL: &str = "floating-card";
/// Logical pixels — within spec section 10.1's recommended 360–420px width.
const CARD_WIDTH: f64 = 380.0;
const SINGLE_CARD_HEIGHT: f64 = 180.0;
/// Batch card needs room for the file list (spec section 9.1's mockup).
const BATCH_CARD_HEIGHT: f64 = 320.0;
const MARGIN: f64 = 16.0;
/// Spec section 10.1 recommended default.
const AUTO_HIDE: Duration = Duration::from_secs(8);

pub const EVENT_SHOW_SINGLE: &str = "floating-card:show";
pub const EVENT_SHOW_BATCH: &str = "floating-card:show-batch";

#[derive(Serialize, Clone)]
struct BatchPayload<'a> {
    files: &'a [FileRecord],
}

/// Bumped on every show so a stale auto-hide timer from an earlier card
/// can't hide a newer one that superseded it.
struct ShowEpoch(AtomicU64);

/// Creates the floating card window: frameless, always-on-top, hidden and
/// unfocused until the first file is ready.
pub fn init(app: &AppHandle) -> tauri::Result<()> {
    WebviewWindowBuilder::new(app, WINDOW_LABEL, WebviewUrl::App("index.html".into()))
        .title("Download Inbox")
        .inner_size(CARD_WIDTH, SINGLE_CARD_HEIGHT)
        .decorations(false)
        .transparent(true)
        .shadow(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .resizable(false)
        .visible(false)
        .focused(false)
        .build()?;

    app.manage(ShowEpoch(AtomicU64::new(0)));
    Ok(())
}

/// Shows the single-file card (spec section 10.2).
pub fn show_single(app: &AppHandle, file: &FileRecord) {
    present(app, SINGLE_CARD_HEIGHT, EVENT_SHOW_SINGLE, file);
}

/// Shows the batch card for 2+ files that became ready close together (spec
/// section 9.1) — the whole point being that this fires once, not once per
/// file.
pub fn show_batch(app: &AppHandle, files: &[FileRecord]) {
    present(
        app,
        BATCH_CARD_HEIGHT,
        EVENT_SHOW_BATCH,
        &BatchPayload { files },
    );
}

/// Positions the card at the bottom-right of the monitor under the cursor,
/// emits `event` with `payload`, shows the window, and schedules an
/// auto-hide. Never calls `set_focus` — per spec section 10.1, only a click
/// on the card itself may focus it.
fn present<P: Serialize + Clone>(app: &AppHandle, height: f64, event: &str, payload: P) {
    let Some(window) = app.get_webview_window(WINDOW_LABEL) else {
        return;
    };

    if let Some(monitor) = target_monitor(app) {
        let scale = monitor.scale_factor();
        let work_area = monitor.work_area();
        let width_px = (CARD_WIDTH * scale).round() as u32;
        let height_px = (height * scale).round() as u32;
        let margin_px = (MARGIN * scale).round() as i32;

        let x = work_area.position.x + work_area.size.width as i32 - width_px as i32 - margin_px;
        let y = work_area.position.y + work_area.size.height as i32 - height_px as i32 - margin_px;

        let _ = window.set_size(Size::Physical(PhysicalSize {
            width: width_px,
            height: height_px,
        }));
        let _ = window.set_position(Position::Physical(PhysicalPosition { x, y }));
    }

    let _ = app.emit_to(WINDOW_LABEL, event, payload);
    let _ = window.show();

    if let Some(epoch_state) = app.try_state::<ShowEpoch>() {
        let epoch = epoch_state.0.fetch_add(1, Ordering::SeqCst) + 1;
        let app = app.clone();
        tauri::async_runtime::spawn(async move {
            tokio::time::sleep(AUTO_HIDE).await;
            if let Some(epoch_state) = app.try_state::<ShowEpoch>() {
                if epoch_state.0.load(Ordering::SeqCst) == epoch {
                    if let Some(window) = app.get_webview_window(WINDOW_LABEL) {
                        let _ = window.hide();
                    }
                }
            }
        });
    }
}

/// The monitor under the cursor (spec: "多显示器时优先跟随鼠标所在显示器"),
/// falling back to the primary monitor if the cursor position is unavailable.
fn target_monitor(app: &AppHandle) -> Option<tauri::window::Monitor> {
    let main = app.get_webview_window("main")?;
    let cursor = main.cursor_position().ok()?;
    main.monitor_from_point(cursor.x, cursor.y)
        .ok()
        .flatten()
        .or_else(|| main.primary_monitor().ok().flatten())
}
