use super::metadata::{self, metadata};
use crate::dprintln;
pub fn init() -> Result<(), std::io::Error> {
    let mut pkg_metadata = metadata().unwrap_or_default();
    let target_dir = metadata::get_dir()?;
    let readme_path = target_dir.join("README.md");
    if readme_path.exists() {
        let readme_content = std::fs::read_to_string(readme_path)?;
        pkg_metadata.about.package.description = readme_content;
        dprintln!("Initialized project metadata.");
    }
    
    Ok(())
}
