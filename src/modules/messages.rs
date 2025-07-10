use crate::utils::shell::{self, markdown};
use std::io;

/// Stores package metadata obtained from Cargo environment variables.
#[derive(Debug, Clone, Copy)]
pub struct CargoPackageInfo {
    /// Package name from Cargo metadata or default
    name: &'static str,
    /// Package version from Cargo metadata or default
    version: &'static str,
    /// Target architecture for the build
    architecture: &'static str,
}

impl CargoPackageInfo {
    /// Creates a new `CargoPackageInfo` instance with package metadata.
    ///
    /// Uses `option_env!` to retrieve Cargo package information at compile time,
    /// falling back to default values if not available.
    pub fn new() -> Self {
        Self {
            name: option_env!("CARGO_PKG_NAME").unwrap_or("ipak"),
            version: option_env!("CARGO_PKG_VERSION").unwrap_or("unknown"),
            architecture: std::env::consts::ARCH,
        }
    }
}

/// Replaces placeholders in a text string with package metadata values.
///
/// # Arguments
/// * `text` - The input text containing placeholders like `{name}`, `{version}`, or `{architecture}`
///
/// # Returns
/// A new `String` with all placeholders replaced with corresponding metadata values
pub fn insert_info(text: &'static str) -> String {
    let cargo_package = CargoPackageInfo::new();
    let replacements = [
        ("{name}", cargo_package.name),
        ("{version}", cargo_package.version),
        ("{architecture}", cargo_package.architecture),
    ];

    let mut result = text.to_string();
    for (placeholder, value) in replacements {
        result = result.replace(placeholder, value);
    }
    result
}

/// Displays the package manual through a pager.
///
/// Reads the manual content from a markdown file, processes it with package metadata,
/// and displays it using the system's pager.
///
/// # Errors
/// Returns an `io::Error` if the pager operation fails.
pub fn manual() -> Result<(), io::Error> {
    let manual_str = markdown(&insert_info(include_str!("./messages/manual.md")));
    shell::pager(manual_str);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_info() {
        let text = "Package: {name}, Version: {version}, Arch: {architecture}";
        let result = insert_info(text);
        assert!(result.contains("ipak") || result.contains(option_env!("CARGO_PKG_NAME").unwrap_or("ipak")));
        assert!(result.contains("unknown") || result.contains(option_env!("CARGO_PKG_VERSION").unwrap_or("unknown")));
        assert!(result.contains(std::env::consts::ARCH));
    }

    #[test]
    fn test_cargo_package_info() {
        let info = CargoPackageInfo::new();
        assert_eq!(info.name, option_env!("CARGO_PKG_NAME").unwrap_or("ipak"));
        assert_eq!(info.version, option_env!("CARGO_PKG_VERSION").unwrap_or("unknown"));
        assert_eq!(info.architecture, std::env::consts::ARCH);
    }
}