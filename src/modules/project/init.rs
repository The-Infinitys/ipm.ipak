use super::super::version::Version;
use super::metadata;
use crate::dprintln;
use crate::utils::files::file_creation; // Added for creating template files
use std::env;
use std::fmt;
use std::fs;
use std::path::Path;
use std::str::FromStr;
enum PackageLanguage {
    Python,
    Rust,
    DotNet,
    Other, // Added Other as a default
}

impl fmt::Display for PackageLanguage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PackageLanguage::Python => write!(f, "python"),
            PackageLanguage::Rust => write!(f, "rust"),
            PackageLanguage::DotNet => write!(f, "dotnet"),
            PackageLanguage::Other => write!(f, "other"), // Handle Other case
        }
    }
}

// Structure to hold template file path and content, similar to create::templates.rs
struct SetUpItem {
    path: String,
    content: String,
}

// Helper function to create files from a list of SetUpItems
fn setup_template_files(setup_list: Vec<SetUpItem>) -> Result<(), std::io::Error> {
    for item in setup_list {
        file_creation(&item.path, &item.content).map_err(|e| {
            std::io::Error::new(
                e.kind(),
                format!("Failed to create template file '{}': {}", item.path, e),
            )
        })?;
    }
    dprintln!("Successfully set up template files.");
    Ok(())
}

pub fn init() -> Result<(), std::io::Error> {
    let mut pkg_metadata = metadata::from_current().unwrap_or_default();
    let target_dir = env::current_dir()?;
    let readme_path = target_dir.join("README.md");

    if readme_path.exists() {
        let readme_content = fs::read_to_string(readme_path)?;
        pkg_metadata.about.package.description = readme_content;
        dprintln!("Initialized project metadata.");
    }

    let mut pkg_lang = PackageLanguage::Other; // Initialize with Other
    let mut lang_file_path_str = String::new(); // Store the path as a String

    if target_dir.join("Cargo.toml").exists() {
        pkg_lang = PackageLanguage::Rust;
        lang_file_path_str =
            target_dir.join("Cargo.toml").to_string_lossy().into_owned();
    } else if target_dir.join("pyproject.toml").exists() {
        pkg_lang = PackageLanguage::Python;
        lang_file_path_str = target_dir
            .join("pyproject.toml")
            .to_string_lossy()
            .into_owned();
    } else {
        // Check for .csproj files recursively
        let dotnet_result = find_csproj_file_recursive(&target_dir)?;
        if let Some(csproj_path) = dotnet_result {
            pkg_lang = PackageLanguage::DotNet;
            lang_file_path_str =
                csproj_path.to_string_lossy().into_owned();
        }
    }

    dprintln!("Detected package language: {}", pkg_lang);

    // Populate name and version based on detected package language
    match pkg_lang {
        PackageLanguage::Rust => {
            if !lang_file_path_str.is_empty() {
                if let Some((name, version)) =
                    parse_cargo_toml(Path::new(&lang_file_path_str))?
                {
                    pkg_metadata.about.package.name = name;
                    pkg_metadata.about.package.version =
                        Version::from_str(&version).unwrap_or_default();
                }
            }
        }
        PackageLanguage::Python => {
            if !lang_file_path_str.is_empty() {
                if let Some((name, version)) =
                    parse_pyproject_toml(Path::new(&lang_file_path_str))?
                {
                    pkg_metadata.about.package.name = name;
                    pkg_metadata.about.package.version =
                        Version::from_str(&version).unwrap_or_default();
                }
            }
        }
        PackageLanguage::DotNet => {
            if !lang_file_path_str.is_empty() {
                if let Some((name, version)) =
                    parse_csproj(Path::new(&lang_file_path_str))?
                {
                    pkg_metadata.about.package.name = name;
                    pkg_metadata.about.package.version =
                        Version::from_str(&version).unwrap_or_default();
                }
            }
        }
        PackageLanguage::Other => {
            dprintln!(
                "No specific package language detected, skipping name and version extraction."
            );
        }
    }
    metadata::to_current(&pkg_metadata)?;
    dprintln!("Project metadata initialized/updated in ipak/project.yaml.");

    dprintln!("Setting up ipak scripts based on detected language...");

    let script_readme_content = include_str!("create/templates/script-README.md").to_string();

    let script_setup_result = match pkg_lang {
        PackageLanguage::Rust => {
            let setup_list = vec![
                SetUpItem {
                    path: "ipak/scripts/build.sh".to_string(),
                    content: include_str!("create/templates/rust/ipak/scripts/build.sh").to_string(),
                },
                SetUpItem {
                    path: "ipak/scripts/install.sh".to_string(),
                    content: include_str!("create/templates/rust/ipak/scripts/install.sh").to_string(),
                },
                SetUpItem {
                    path: "ipak/scripts/remove.sh".to_string(),
                    content: include_str!("create/templates/rust/ipak/scripts/remove.sh").to_string(),
                },
                SetUpItem {
                    path: "ipak/scripts/purge.sh".to_string(),
                    content: include_str!("create/templates/rust/ipak/scripts/purge.sh").to_string(),
                },
                SetUpItem {
                    path: "ipak/project-ignore.yaml".to_string(),
                    content: include_str!("create/templates/rust/ipak/project-ignore.yaml").to_string(),
                },
                SetUpItem {
                    path: "ipak/scripts/README.md".to_string(),
                    content: script_readme_content.clone(),
                },
            ];
            setup_template_files(setup_list)
        }
        PackageLanguage::Python => {
            let setup_list = vec![
                SetUpItem {
                    path: "ipak/scripts/build.sh".to_string(),
                    content: include_str!("create/templates/python/ipak/scripts/build.sh").to_string(),
                },
                SetUpItem {
                    path: "ipak/scripts/install.sh".to_string(),
                    content: include_str!("create/templates/python/ipak/scripts/install.sh").to_string(),
                },
                SetUpItem {
                    path: "ipak/scripts/remove.sh".to_string(),
                    content: include_str!("create/templates/python/ipak/scripts/remove.sh").to_string(),
                },
                SetUpItem {
                    path: "ipak/scripts/purge.sh".to_string(),
                    content: include_str!("create/templates/python/ipak/scripts/purge.sh").to_string(),
                },
                SetUpItem {
                    path: "ipak/scripts/README.md".to_string(),
                    content: script_readme_content.clone(),
                },
            ];
            setup_template_files(setup_list)
        }
        PackageLanguage::DotNet => {
            let setup_list = vec![
                SetUpItem {
                    path: "ipak/scripts/build.sh".to_string(),
                    content: include_str!("create/templates/dotnet/ipak/scripts/build.sh").to_string(),
                },
                SetUpItem {
                    path: "ipak/scripts/install.sh".to_string(),
                    content: include_str!("create/templates/dotnet/ipak/scripts/install.sh").to_string(),
                },
                SetUpItem {
                    path: "ipak/scripts/remove.sh".to_string(),
                    content: include_str!("create/templates/dotnet/ipak/scripts/remove.sh").to_string(),
                },
                SetUpItem {
                    path: "ipak/scripts/purge.sh".to_string(),
                    content: include_str!("create/templates/dotnet/ipak/scripts/purge.sh").to_string(),
                },
                SetUpItem {
                    path: "ipak/project-ignore.yaml".to_string(),
                    content: include_str!("create/templates/dotnet/ipak/project-ignore.yaml").to_string(),
                },
                SetUpItem {
                    path: "ipak/scripts/README.md".to_string(),
                    content: script_readme_content.clone(),
                },
            ];
            setup_template_files(setup_list)
        }
        PackageLanguage::Other => { // Default scripts
            let setup_list = vec![
                SetUpItem {
                    path: "ipak/scripts/build.sh".to_string(),
                    content: include_str!("create/templates/default/ipak/scripts/build.sh").to_string(),
                },
                SetUpItem {
                    path: "ipak/scripts/install.sh".to_string(),
                    content: include_str!("create/templates/default/ipak/scripts/install.sh").to_string(),
                },
                SetUpItem {
                    path: "ipak/scripts/remove.sh".to_string(),
                    content: include_str!("create/templates/default/ipak/scripts/remove.sh").to_string(),
                },
                SetUpItem {
                    path: "ipak/scripts/purge.sh".to_string(),
                    content: include_str!("create/templates/default/ipak/scripts/purge.sh").to_string(),
                },
                SetUpItem {
                    path: "ipak/scripts/README.md".to_string(),
                    content: script_readme_content, // No clone needed for last use
                },
            ];
            setup_template_files(setup_list)
        }
    };

    script_setup_result?; // Propagate any errors from script setup

    dprintln!("ipak init process completed successfully.");
    Ok(())
}

/// Helper function to recursively find .csproj files
fn find_csproj_file_recursive(
    dir: &Path,
) -> Result<Option<std::path::PathBuf>, std::io::Error> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            if path.extension().is_some_and(|ext| ext == "csproj") {
                return Ok(Some(path));
            }
        } else if path.is_dir() {
            // Avoid recursing into common dependency directories
            if path.file_name().is_some_and(|name| {
                name == "target"
                    || name == "node_modules"
                    || name == "bin"
                    || name == "obj"
            }) {
                continue;
            }
            if let Some(csproj_path) = find_csproj_file_recursive(&path)? {
                return Ok(Some(csproj_path));
            }
        }
    }
    Ok(None)
}

// Helper function to parse Cargo.toml for name and version
fn parse_cargo_toml(
    path: &Path,
) -> Result<Option<(String, String)>, std::io::Error> {
    let content = fs::read_to_string(path)?;
    if let Ok(toml_doc) = content.parse::<toml::Value>() {
        if let Some(package) = toml_doc.get("package") {
            let name = package
                .get("name")
                .and_then(|n| n.as_str())
                .map(|s| s.to_string());
            let version = package
                .get("version")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            if let (Some(name), Some(version)) = (name, version) {
                return Ok(Some((name, version)));
            }
        }
    }
    Ok(None)
}

// Helper function to parse pyproject.toml for name and version
fn parse_pyproject_toml(
    path: &Path,
) -> Result<Option<(String, String)>, std::io::Error> {
    let content = fs::read_to_string(path)?;
    if let Ok(toml_doc) = content.parse::<toml::Value>() {
        if let Some(project) = toml_doc.get("project") {
            let name = project
                .get("name")
                .and_then(|n| n.as_str())
                .map(|s| s.to_string());
            let version = project
                .get("version")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            if let (Some(name), Some(version)) = (name, version) {
                return Ok(Some((name, version)));
            }
        }
    }
    Ok(None)
}

// Helper function to parse .csproj for AssemblyName and Version
fn parse_csproj(
    path: &Path,
) -> Result<Option<(String, String)>, std::io::Error> {
    let content = fs::read_to_string(path)?;
    // A very basic XML parsing for AssemblyName and Version
    // This could be improved with a proper XML parsing library for robustness
    let name_tag_start = "<AssemblyName>";
    let name_tag_end = "</AssemblyName>";
    let version_tag_start = "<Version>";
    let version_tag_end = "</Version>";

    let mut name: Option<String> = None;
    let mut version: Option<String> = None;

    if let Some(start) = content.find(name_tag_start) {
        if let Some(end) = content[start..].find(name_tag_end) {
            name = Some(
                content[start + name_tag_start.len()..start + end]
                    .trim()
                    .to_string(),
            );
        }
    }

    if let Some(start) = content.find(version_tag_start) {
        if let Some(end) = content[start..].find(version_tag_end) {
            version = Some(
                content[start + version_tag_start.len()..start + end]
                    .trim()
                    .to_string(),
            );
        }
    }

    if let (Some(name_val), Some(version_val)) = (name, version) {
        Ok(Some((name_val, version_val)))
    } else {
        Ok(None)
    }
}
