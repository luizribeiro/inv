use std::path::Path;

use crate::error::Result;

pub fn run(db_path: &Path) -> Result<()> {
    crate::storage::load_inventory(db_path).map(|_| ())
}
