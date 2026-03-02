use std::fs::OpenOptions;
use std::io::{ErrorKind, Write};
use std::path::Path;

use crate::error::{AppError, Result};
use crate::model::InventoryDoc;

pub fn run(db_path: &Path) -> Result<()> {
    let mut json = serde_json::to_string_pretty(&InventoryDoc::default()).map_err(|error| {
        AppError::Validation(format!(
            "failed to serialize default inventory document: {error}"
        ))
    })?;
    json.push('\n');

    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(db_path)
        .map_err(|source| {
            if source.kind() == ErrorKind::AlreadyExists {
                AppError::Validation(format!(
                    "inventory file '{}' already exists; refusing to overwrite",
                    db_path.display()
                ))
            } else {
                AppError::Io {
                    path: db_path.to_path_buf(),
                    action: "create inventory file",
                    source,
                }
            }
        })?;

    file.write_all(json.as_bytes())
        .map_err(|source| AppError::Io {
            path: db_path.to_path_buf(),
            action: "write inventory file",
            source,
        })?;

    file.sync_all().map_err(|source| AppError::Io {
        path: db_path.to_path_buf(),
        action: "fsync inventory file",
        source,
    })?;

    Ok(())
}
