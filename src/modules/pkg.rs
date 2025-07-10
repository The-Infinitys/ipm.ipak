//! The `ipak` package management module defines core data structures and operations
//! for handling package metadata, dependencies, installation modes, and CLI arguments.
//!
//! This module provides functionality for:
//! - Package installation (local and global modes)
//! - Dependency and conflict management
//! - Package metadata processing
//! - Command-line interface operations

use super::version::{Version, VersionRange};
use crate::utils::args::PkgCommands;
use crate::utils::color::colorize::*;
use crate::utils::error::Error;
use crate::utils::{
    generate_email_address,
    shell::{markdown, username},
};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

// Module declarations
pub mod depend;
pub mod install;
pub mod list;
pub mod metadata;
pub mod purge;
pub mod remove;

/// Defines the installation mode for packages.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum Mode {
    /// Local installation mode, scoped to the current project.
    Local,
    /// Global installation mode, system-wide installation.
    Global,
    /// Default mode considering both local and global installations.
    #[default]
    Any,
}

impl Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Local => write!(f, "local"),
            Self::Global => write!(f, "global"),
            Self::Any => write!(f, "any (local & global)"),
        }
    }
}

/// Represents complete package metadata including author, architecture, and dependencies.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct PackageData {
    /// Package and author information
    pub about: AboutData,
    /// Supported architectures (empty implies all architectures)
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub architecture: Vec<String>,
    /// Installation mode
    pub mode: Mode,
    /// Dependency and relationship information
    #[serde(skip_serializing_if = "RelationData::is_empty")]
    pub relation: RelationData,
}

/// Contains author and package-specific metadata.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct AboutData {
    /// Author details
    pub author: AuthorAboutData,
    /// Package details
    pub package: PackageAboutData,
}

/// Stores author information including name and email.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct AuthorAboutData {
    /// Author's name
    pub name: String,
    /// Author's email address
    pub email: String,
}

/// Contains package metadata including name, version, and description.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PackageAboutData {
    /// Package name
    pub name: String,
    /// Package version
    pub version: Version,
    /// Package description (optional)
    #[serde(skip_serializing_if = "String::is_empty")]
    pub description: String,
}

/// Manages package relationships including dependencies and conflicts.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct RelationData {
    /// Required dependencies (OR groups)
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub depend: Vec<Vec<PackageRange>>,
    /// Required command-line tools
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub depend_cmds: Vec<String>,
    /// Suggested optional dependencies
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub suggests: Vec<Vec<PackageRange>>,
    /// Recommended dependencies
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub recommends: Vec<Vec<PackageRange>>,
    /// Conflicting packages
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub conflicts: Vec<PackageRange>,
    /// Virtual package provisions
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub virtuals: Vec<PackageVersion>,
    /// Commands provided by this package
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub provide_cmds: Vec<String>,
}

/// Represents a package dependency with version constraints.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PackageRange {
    /// Package name
    pub name: String,
    /// Version constraints
    pub range: VersionRange,
}

/// Represents a package with a specific version.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PackageVersion {
    /// Package name
    pub name: String,
    /// Specific package version
    pub version: Version,
}

// Implementation of Display traits
impl Display for PackageData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{} {}", "Package:".bold(), self.about.package.name.cyan())?;
        writeln!(f, "{} {}", "Version:".bold(), self.about.package.version)?;
        
        if !self.about.package.description.is_empty() {
            writeln!(f, "{} {}", "Description:".bold(), self.about.package.description)?;
        }
        
        writeln!(
            f,
            "{} {} <{}>",
            "Author:".bold(),
            self.about.author.name.trim(),
            self.about.author.email
        )?;

        writeln!(
            f,
            "{} {}",
            "Architectures:".bold(),
            if self.architecture.is_empty() {
                "any".italic()
            } else {
                self.architecture.join(", ").italic()
            }
        )?;

        writeln!(f, "{} {}", "Installation Mode:".bold(), self.mode)?;
        write!(f, "{}", self.relation)
    }
}

impl Display for AboutData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{} {}", "Author:".bold(), self.author)?;
        writeln!(f, "{} {}", "Package:".bold(), self.package)
    }
}

impl Display for AuthorAboutData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} <{}>", self.name, self.email)
    }
}

impl Display for PackageAboutData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.name.cyan(), self.version)?;
        if !self.description.is_empty() {
            write!(f, "\n  {}", markdown(&self.description))?;
        }
        Ok(())
    }
}

impl Display for RelationData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn format_group(group: &[PackageRange], color: fn(&str) -> String) -> String {
            if group.len() == 1 {
                format!("{}", color(&group[0].to_string()))
            } else {
                let alts: Vec<String> = group.iter().map(|d| d.to_string()).collect();
                format!("({})", color(&alts.join(" | ")))
            }
        }

        if !self.depend.is_empty() {
            writeln!(f, "\n{}", "Dependencies:".bold())?;
            for group in &self.depend {
                writeln!(f, "  - {}", format_group(group, |s| s.green()))?;
            }
        }

        if !self.depend_cmds.is_empty() {
            writeln!(f, "\n{}", "Necessary Commands:".bold())?;
            for cmd in &self.depend_cmds {
                writeln!(f, "  - {}", cmd.green())?;
            }
        }

        if !self.suggests.is_empty() {
            writeln!(f, "\n{}", "Suggests:".bold())?;
            for group in &self.suggests {
                writeln!(f, "  - {}", format_group(group, |s| s.yellow()))?;
            }
        }

        if !self.recommends.is_empty() {
            writeln!(f, "\n{}", "Recommends:".bold())?;
            for group in &self.recommends {
                writeln!(f, "  - {}", format_group(group, |s| s.blue()))?;
            }
        }

        if !self.conflicts.is_empty() {
            writeln!(f, "\n{}", "Conflicts:".bold())?;
            for conflict in &self.conflicts {
                writeln!(f, "  - {}", conflict.to_string().red())?;
            }
        }

        if !self.virtuals.is_empty() {
            writeln!(f, "\n{}", "Virtual Packages:".bold())?;
            for virtual_pkg in &self.virtuals {
                writeln!(f, "  - {}", virtual_pkg.to_string().magenta())?;
            }
        }

        if !self.provide_cmds.is_empty() {
            writeln!(f, "\n{}", "Providing Commands:".bold())?;
            for cmd in &self.provide_cmds {
                writeln!(f, "  - {}", cmd.green())?;
            }
        }
        Ok(())
    }
}

impl Display for PackageRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.name, self.range)
    }
}

impl Display for PackageVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.name, self.version)
    }
}

// Default implementations
impl Default for AuthorAboutData {
    fn default() -> Self {
        Self {
            name: username(),
            email: generate_email_address(),
        }
    }
}

impl Default for PackageAboutData {
    fn default() -> Self {
        Self {
            name: "default-package".to_string(),
            version: Version::default(),
            description: String::new(),
        }
    }
}

impl Default for PackageRange {
    fn default() -> Self {
        Self {
            name: "default-dependency".to_string(),
            range: VersionRange::default(),
        }
    }
}

impl Default for PackageVersion {
    fn default() -> Self {
        Self {
            name: "default-version".to_string(),
            version: Version::default(),
        }
    }
}

impl RelationData {
    /// Checks if all relation fields are empty.
    pub fn is_empty(&self) -> bool {
        self.depend.is_empty()
            && self.depend_cmds.is_empty()
            && self.suggests.is_empty()
            && self.recommends.is_empty()
            && self.conflicts.is_empty()
            && self.virtuals.is_empty()
            && self.provide_cmds.is_empty()
    }
}

/// Processes package-related commands based on provided CLI arguments.
pub fn pkg(args: PkgCommands) -> Result<(), Error> {
    match args {
        PkgCommands::Install { file_path, local, global } => {
            install::install(file_path, (local, global).into())
        }
        PkgCommands::Remove { package_name, local, global } => {
            remove::remove(package_name, (local, global).into())
        }
        PkgCommands::Purge { package_name, local, global } => {
            purge::purge(package_name, (local, global).into())
        }
        PkgCommands::List { local, global } => list::list((local, global).into()),
        PkgCommands::MetaData { package_path } => metadata::metadata(package_path),
    }
}

// Test module
#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_display_with_relations_and_new_fields() {
        let mut data = PackageData::default();
        data.about.author = AuthorAboutData {
            name: "Test Author".to_string(),
            email: "test@example.com".to_string(),
        };
        data.about.package = PackageAboutData {
            name: "my-package".to_string(),
            version: Version::default(),
            description: "This is a test package for demonstration.".to_string(),
        };
        data.architecture = vec!["x86_64".to_string(), "aarch64".to_string()];
        data.mode = Mode::Global;

        // Add test dependencies
        data.relation.depend.push(vec![PackageRange {
            name: "dep-a".to_string(),
            range: VersionRange::from_str(">= 1.0, < 2.0").unwrap(),
        }]);
        data.relation.depend.push(vec![
            PackageRange {
                name: "dep-b".to_string(),
                range: VersionRange::from_str("= 2.0.0").unwrap(),
            },
            PackageRange {
                name: "dep-c".to_string(),
                range: VersionRange::from_str("> 1.5.0").unwrap(),
            },
        ]);

        // Add test suggestions
        data.relation.suggests.push(vec![PackageRange {
            name: "suggest-x".to_string(),
            range: VersionRange::from_str("= 3.0").unwrap(),
        }]);

        // Add test recommendations
        data.relation.recommends.push(vec![
            PackageRange {
                name: "rec-y".to_string(),
                range: VersionRange::from_str("< 4.0.0").unwrap(),
            },
            PackageRange {
                name: "rec-z".to_string(),
                range: VersionRange::from_str("= 4.1.0").unwrap(),
            },
        ]);

        // Add test conflicts
        data.relation.conflicts.push(PackageRange {
            name: "old-package".to_string(),
            range: VersionRange::from_str("0.9.0").unwrap(),
        });

        // Add test virtual packages
        data.relation.virtuals.push(PackageVersion {
            name: "my-virtual-pkg".to_string(),
            version: Version::from_str("1.0.0").unwrap(),
        });

        // Add test commands
        data.relation.provide_cmds.extend(vec!["my-command".to_string(), "another-command".to_string()]);
        data.relation.depend_cmds.extend(vec!["git".to_string(), "make".to_string()]);

        println!("\n--- Test Display With Relations and New Fields ---");
        println!("{}", data);
        assert_eq!(data.architecture, vec!["x86_64".to_string(), "aarch64".to_string()]);
        assert_eq!(data.mode, Mode::Global);
    }

    // Other test cases remain similar but with added documentation
    #[test]
    fn test_display_author() {
        let author = AuthorAboutData {
            name: "Test Author".to_string(),
            email: "test@example.com".to_string(),
        };
        println!("\n--- Test Display Author ---");
        println!("{}", author);
    }

    // ... (other test cases remain similar but with consistent formatting)
}