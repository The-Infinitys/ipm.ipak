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
    // let clang_meta_path=target_dir.join("CMakeLists.txt");
    let rust_metadata_path = target_dir.join("Cargo.toml");
    let python_metadata_path = target_dir.join("pyproject.toml");
    let dotnet_
    Ok(())
}
