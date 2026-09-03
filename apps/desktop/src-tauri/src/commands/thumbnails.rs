//! Gallery thumbnails. Not a `file-operations` concern — it's read-only, no
//! mutation, so it doesn't go through the preflight/log/execute pipeline
//! spec section 18 requires for the four mutating operations.
//!
//! Returns a base64 data URI over normal command JSON rather than using
//! Tauri's asset-protocol (`convertFileSrc`): that protocol's `scope` is a
//! static allow-list in `tauri.conf.json`, but Groups point at folders the
//! user picks at runtime — there's no fixed set of paths to allow-list
//! ahead of time.

use base64::engine::general_purpose::STANDARD;
use base64::Engine;

const MAX_DIMENSION: u32 = 320;

#[tauri::command]
pub async fn get_thumbnail(path: String) -> Result<String, String> {
    tokio::task::spawn_blocking(move || generate(&path))
        .await
        .map_err(|err| err.to_string())?
}

fn generate(path: &str) -> Result<String, String> {
    let img = image::open(path).map_err(|err| err.to_string())?;
    let thumb = img.thumbnail(MAX_DIMENSION, MAX_DIMENSION);

    let mut bytes: Vec<u8> = Vec::new();
    thumb
        .write_to(
            &mut std::io::Cursor::new(&mut bytes),
            image::ImageFormat::Png,
        )
        .map_err(|err| err.to_string())?;

    Ok(format!("data:image/png;base64,{}", STANDARD.encode(&bytes)))
}
