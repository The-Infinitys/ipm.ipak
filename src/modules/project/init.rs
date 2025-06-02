use super::metadata::{self, metadata};
use crate::dprintln;
pub fn init() -> Result<(), std::io::Error> {
    let pkg_metadata = metadata().unwrap_or_default();
    let target_dir = metadata::get_dir()?;
    dprintln!("{}\n{}", pkg_metadata, target_dir.display());
    Ok(())
}
