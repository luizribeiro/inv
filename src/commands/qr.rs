use std::path::{Path, PathBuf};

use qrcode::QrCode;
use uuid::Uuid;

use crate::error::{AppError, Result};

pub fn run(db_path: &Path, id: &str, out: Option<&Path>) -> Result<()> {
    let item_id = Uuid::parse_str(id)
        .map_err(|_| AppError::Validation(format!("invalid item id '{id}' (expected UUID)")))?;

    let doc = crate::storage::load_inventory(db_path)?;
    let item = doc
        .items
        .iter()
        .find(|candidate| candidate.id == item_id)
        .ok_or_else(|| AppError::Validation(format!("item '{item_id}' not found")))?;

    let output_path = out
        .map(Path::to_path_buf)
        .unwrap_or_else(|| default_output_path(item.id));

    let qr = QrCode::new(item.id.as_hyphenated().to_string().as_bytes())
        .map_err(|error| AppError::Validation(format!("failed to generate QR code: {error}")))?;

    let image = qr
        .render::<image::Luma<u8>>()
        .min_dimensions(256, 256)
        .build();

    image.save(&output_path).map_err(|error| {
        AppError::Validation(format!(
            "failed to save QR image to '{}': {error}",
            output_path.display()
        ))
    })?;

    println!(
        "Wrote QR code for item {} to {}",
        item.id.as_hyphenated(),
        output_path.display()
    );

    Ok(())
}

fn default_output_path(id: Uuid) -> PathBuf {
    PathBuf::from(format!("{}.png", id.as_hyphenated()))
}
