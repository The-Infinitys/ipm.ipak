use super::metadata;
use crate::dprintln;
use crate::utils::archive::{ArchiveType, create_archive}; // archive.rsからインポート
use crate::utils::color::colorize::*;
use ignore::gitignore::GitignoreBuilder;
use serde_yaml;
use std::fmt::{self, Display};
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

/// Defines the options for the packaging process.
#[derive(Debug, Default)]
pub struct PackageOptions {
    /// The target type for the package (e.g., source build, normal, minimal).
    pub target: PackageTarget,
}

/// Represents the different packaging targets.
#[derive(Debug, Clone, Copy, Default)]
pub enum PackageTarget {
    /// Builds from source.
    SourceBuild,
    /// Standard package.
    #[default]
    Normal,
    /// Minimal package.
    Min,
}

impl Display for PackageTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PackageTarget::SourceBuild => write!(f, "source-build"),
            PackageTarget::Normal => write!(f, "normal"),
            PackageTarget::Min => write!(f, "minimal"),
        }
    }
}

impl FromStr for PackageTarget {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "src" | "source" | "source-build" => Ok(Self::SourceBuild),
            "normal" | "default" => Ok(Self::Normal),
            "min" | "minimal" => Ok(Self::Min),
            _ => Err(format!("Invalid Package Target: {}", s)),
        }
    }
}

impl Display for PackageOptions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}:", "Package Options".cyan().bold())?;
        writeln!(f, "  {}: {}", "target".green().bold(), self.target)?;
        Ok(())
    }
}

/// Structure to deserialize project-ignore.yaml
#[derive(serde::Deserialize)]
struct ProjectIgnore {
    #[serde(rename = "source-build")]
    source_build: Vec<String>,
    normal: Vec<String>,
    min: Vec<String>,
}

/// Function to recursively walk and copy files from source to destination,
/// while respecting the ignore list and skipping specified prefixes.
fn walk_and_copy(
    source_base: &Path,
    dest_base: &Path,
    gitignore: &ignore::gitignore::Gitignore,
    skip_prefix: &Path,
) -> Result<(), String> {
    fn inner(
        dir: &Path,
        source_base: &Path,
        dest_base: &Path,
        gitignore: &ignore::gitignore::Gitignore,
        skip_prefix: &Path,
    ) -> Result<(), String> {
        let dir_rel = dir.strip_prefix(source_base).map_err(|_| {
            format!("Failed to get relative path for directory {:?}", dir)
        })?;
        if dir_rel.starts_with(skip_prefix) {
            return Ok(());
        }

        for entry in fs::read_dir(dir).map_err(|e| {
            format!("Failed to read directory {:?}: {}", dir, e)
        })? {
            let entry = entry
                .map_err(|e| format!("Failed to get entry: {}", e))?;
            let path = entry.path();
            let path_rel =
                path.strip_prefix(source_base).map_err(|_| {
                    format!("Failed to get relative path for {:?}", path)
                })?;

            if path.is_dir() {
                inner(
                    &path,
                    source_base,
                    dest_base,
                    gitignore,
                    skip_prefix,
                )?;
            } else {
                if gitignore.matched(path_rel, true).is_ignore() {
                    dprintln!("Ignored: {}", path_rel.display());
                } else {
                    let dest = dest_base.join(path_rel);
                    if let Some(parent) = dest.parent() {
                        fs::create_dir_all(parent).map_err(|e| {
                            format!(
                                "Failed to create directories for {:?}: {}",
                                parent, e
                            )
                        })?;
                    }
                    fs::copy(&path, &dest).map_err(|e| {
                        format!(
                            "Failed to copy {:?} to {:?}: {}",
                            path, dest, e
                        )
                    })?;
                    dprintln!(
                        "Copied {} to {}",
                        path.display(),
                        dest.display()
                    );
                }
            }
        }

        Ok(())
    }

    inner(source_base, source_base, dest_base, gitignore, skip_prefix)
}

/// Initiates the packaging process based on the provided options.
pub fn package(opts: PackageOptions) -> Result<(), String> {
    dprintln!("Starting packaging process with options: {}", &opts);

    // Get project directory
    let target_dir = metadata::get_dir().map_err(|e| {
        format!(
            "Error: Couldn't find Ipak Directory. Make sure you are in an ipak project. Details: {:?}", 
            e
        )
    })?;
    dprintln!("Project directory: {}", target_dir.display());

    // Load project metadata
    let project_metadata = metadata::metadata().map_err(|e| {
        format!("Error: Failed to read project metadata: {:?}", e)
    })?;
    dprintln!(
        "Project metadata loaded for: {} version {}",
        project_metadata.about.package.name,
        project_metadata.about.package.version
    );

    // Load project-ignore.yaml
    let ignore_file = target_dir.join("ipak").join("project-ignore.yaml");
    let ignore_config: ProjectIgnore = if ignore_file.exists() {
        let file = fs::File::open(&ignore_file).map_err(|e| {
            format!("Failed to open '{}': {}", ignore_file.display(), e)
        })?;
        serde_yaml::from_reader(file).map_err(|e| {
            format!("Failed to parse '{}': {}", ignore_file.display(), e)
        })?
    } else {
        dprintln!(
            "Warning: '{}' not found, using empty ignore lists",
            ignore_file.display()
        );
        ProjectIgnore { source_build: vec![], normal: vec![], min: vec![] }
    };

    // Select ignore list based on PackageTarget
    let ignore_list: Vec<String> = match opts.target {
        PackageTarget::SourceBuild => ignore_config.source_build,
        PackageTarget::Normal => {
            let mut list = ignore_config.source_build;
            list.extend(ignore_config.normal);
            list
        }
        PackageTarget::Min => {
            let mut list = ignore_config.source_build;
            list.extend(ignore_config.normal);
            list.extend(ignore_config.min);
            list
        }
    };

    dprintln!(
        "Ignore list for target {}: [\n{}\n]",
        opts.target,
        ignore_list.join("\n")
    );
    dprintln!("Target Directory: {}", target_dir.display());

    // Build Gitignore from the ignore list
    let mut builder = GitignoreBuilder::new(&target_dir);
    for pattern in &ignore_list {
        if let Err(e) = builder.add_line(None, pattern.as_str()) {
            eprintln!("Error: {}", e)
        };
        dprintln!("Adding ignore pattern: {}", pattern);
    }
    let gitignore = builder
        .build()
        .map_err(|e| format!("Failed to build gitignore: {}", e))?;
    dprintln!("Gitignore built: {}", gitignore.len());

    // Define source base, destination base, and skip prefix
    let source_base = &target_dir;
    let package_name = &project_metadata.about.package.name;
    let version = &project_metadata.about.package.version;
    let dest_base: PathBuf = source_base
        .join("ipak")
        .join("package")
        .join(format!("{}-{}", package_name, version));
    let skip_prefix: PathBuf = PathBuf::from("ipak").join("package");

    // Call walk_and_copy to perform the file copying
    walk_and_copy(source_base, &dest_base, &gitignore, &skip_prefix)?;

    // Compress the destination directory into a tar.gz archive
    let archive_path: PathBuf = source_base
        .join("ipak")
        .join("package")
        .join(format!("{}-{}.tar.gz", package_name, version)); // .ipakを.tar.gzに変更
    // 注: .ipakを保持したい場合、拡張子を"{}-{}.ipak"に変更し、ArchiveType::TarGzを明示的に使用

    // Ensure the parent directory of the archive exists
    if let Some(parent) = archive_path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            format!("Failed to create directories for {:?}: {}", parent, e)
        })?;
    }

    // create_archiveを使用してTarGzアーカイブを作成
    dprintln!("Creating tar.gz archive at {}", archive_path.display());
    create_archive(
        dest_base.clone(),
        archive_path.clone(),
        ArchiveType::TarGz,
    )
    .map_err(|e| format!("Failed to create archive: {}", e))?;

    // Clean up the destination directory after compression
    fs::remove_dir_all(&dest_base).map_err(|e| {
        format!("Failed to remove directory {:?}: {}", dest_base, e)
    })?;
    dprintln!("Removed temporary directory {}", dest_base.display());

    // Verify the archive exists
    if !archive_path.exists() {
        return Err(format!(
            "Archive file {} was not created",
            archive_path.display()
        ));
    }

    dprintln!("Created archive at {}", archive_path.display());

    Ok(())
}
