use super::messages;
use super::version::{Version, VersionRange};
use crate::utils::color::colorize::*;
use crate::utils::{
    generate_email_address, shell::markdown, shell::username,
};
use cmd_arg::cmd_arg::Option;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
pub mod depend;
mod install;
pub mod list;
mod metadata;
mod purge;
mod remove;
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum Mode {
    Local,
    Global,
    #[default]
    Any,
}

impl Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Mode::Local => write!(f, "local"),
            Mode::Global => write!(f, "global"),
            Mode::Any => {
                write!(f, "any (local & global)")
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
#[derive(Default)]
pub struct PackageData {
    pub about: AboutData,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub architecture: Vec<String>,
    pub mode: Mode,
    #[serde(skip_serializing_if = "RelationData::is_empty")]
    pub relation: RelationData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
#[derive(Default)]
pub struct AboutData {
    pub author: AuthorAboutData,
    pub package: PackageAboutData,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct AuthorAboutData {
    pub name: String,
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PackageAboutData {
    pub name: String,
    pub version: Version,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub description: String, // Added description field
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
#[derive(Default)]
pub struct RelationData {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub depend: Vec<Vec<PackageRange>>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub depend_cmds: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub suggests: Vec<Vec<PackageRange>>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub recommends: Vec<Vec<PackageRange>>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub conflicts: Vec<PackageRange>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub virtuals: Vec<PackageVersion>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub provide_cmds: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PackageRange {
    pub name: String,
    pub range: VersionRange,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PackageVersion {
    pub name: String,
    pub version: Version,
}

impl Display for PackageData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "{} {}",
            "Package:".bold(),
            self.about.package.name.cyan()
        )?;
        writeln!(
            f,
            "{} {}",
            "Version:".bold(),
            self.about.package.version
        )?;
        if !self.about.package.description.is_empty() {
            writeln!(
                f,
                "{} {}",
                "Description:".bold(),
                self.about.package.description
            )?;
        }
        writeln!(
            f,
            "{} {} <{}>",
            "Author:".bold(),
            self.about.author.name.trim(),
            self.about.author.email
        )?;

        if !&self.architecture.is_empty() {
            writeln!(
                f,
                "{} {}",
                "Architectures:".bold(),
                &self.architecture.join(", ").italic()
            )?;
        } else {
            writeln!(f, "{} {}", "Architectures:".bold(), "any".italic())?;
        }
        writeln!(
            f,
            "{} {}",
            "Available Installation Mode:".bold(),
            self.mode
        )?;

        if !self.relation.depend.is_empty() {
            writeln!(f, "\n{}", "Dependencies:".bold())?;
            for group in &self.relation.depend {
                if group.len() == 1 {
                    let dep = &group[0];
                    writeln!(
                        f,
                        "  - {} ({})",
                        dep.name.green(),
                        dep.range
                    )?;
                } else {
                    let alts: Vec<String> = group
                        .iter()
                        .map(|d| format!("{} ({})", d.name, d.range))
                        .collect();
                    let alts_str = alts.join(" | ");
                    writeln!(f, "  - ({})", alts_str.green())?;
                }
            }
        }

        if !self.relation.depend_cmds.is_empty() {
            writeln!(f, "\n{}", "Necessary Commands:".bold())?;
            for cmd in &self.relation.depend_cmds {
                writeln!(f, "  - {}", cmd.green())?;
            }
        }

        if !self.relation.suggests.is_empty() {
            writeln!(f, "\n{}", "Suggests:".bold())?;
            for group in &self.relation.suggests {
                if group.len() == 1 {
                    let dep = &group[0];
                    writeln!(
                        f,
                        "  - {} ({})",
                        dep.name.yellow(),
                        dep.range
                    )?;
                } else {
                    let alts: Vec<String> = group
                        .iter()
                        .map(|d| format!("{} ({})", d.name, d.range))
                        .collect();
                    let alts_str = alts.join(" | ");
                    writeln!(f, "  - ({})", alts_str.yellow())?;
                }
            }
        }

        if !self.relation.recommends.is_empty() {
            writeln!(f, "\n{}", "Recommends:".bold())?;
            for group in &self.relation.recommends {
                if group.len() == 1 {
                    let dep = &group[0];
                    writeln!(
                        f,
                        "  - {} ({})",
                        dep.name.blue(),
                        dep.range
                    )?;
                } else {
                    let alts: Vec<String> = group
                        .iter()
                        .map(|d| format!("{} ({})", d.name, d.range))
                        .collect();
                    let alts_str = alts.join(" | ");
                    writeln!(f, "  - ({})", alts_str.blue())?;
                }
            }
        }

        if !self.relation.conflicts.is_empty() {
            writeln!(f, "\n{}", "Conflicts:".bold())?;
            for conflict in &self.relation.conflicts {
                writeln!(
                    f,
                    "  - {} ({})",
                    conflict.name.red(),
                    conflict.range
                )?;
            }
        }

        if !self.relation.virtuals.is_empty() {
            writeln!(f, "\n{}", "Virtual Packages:".bold())?;
            for virtual_pkg in &self.relation.virtuals {
                writeln!(
                    f,
                    "  - {} ({})",
                    virtual_pkg.name.magenta(),
                    virtual_pkg.version
                )?;
            }
        }

        if !self.relation.provide_cmds.is_empty() {
            writeln!(f, "\n{}", "Providing Commands:".bold())?;
            for cmd in &self.relation.provide_cmds {
                writeln!(f, "  - {}", cmd.green())?;
            }
        }
        Ok(())
    }
}

impl Display for AboutData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{} {}", "Author:".bold(), self.author)?;
        writeln!(f, "{} {}", "Package:".bold(), self.package)?;
        Ok(())
    }
}

impl Display for AuthorAboutData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} <{}>", self.name, self.email)?;
        Ok(())
    }
}

impl Display for PackageAboutData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.name.cyan(), self.version)?;
        if !self.description.is_empty() {
            write!(f, "\n  {}", markdown(self.description.to_string()))?; // Indent for better readability
        }
        Ok(())
    }
}

impl Display for RelationData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.depend.is_empty() {
            writeln!(f, "{}", "Dependencies:".bold())?;
            for group in &self.depend {
                if group.len() == 1 {
                    writeln!(f, "  - {}", &group[0])?;
                } else {
                    let alts: Vec<String> =
                        group.iter().map(|d| d.to_string()).collect();
                    let alts_str = alts.join(" | ");
                    writeln!(f, "  - ({})", alts_str.green())?;
                }
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
                if group.len() == 1 {
                    writeln!(f, "  - {}", group[0].to_string().yellow())?;
                } else {
                    let alts: Vec<String> =
                        group.iter().map(|d| d.to_string()).collect();
                    let alts_str = alts.join(" | ");
                    writeln!(f, "  - ({})", alts_str.yellow())?;
                }
            }
        }

        if !self.recommends.is_empty() {
            writeln!(f, "\n{}", "Recommends:".bold())?;
            for group in &self.recommends {
                if group.len() == 1 {
                    writeln!(f, "  - {}", group[0].to_string().blue())?;
                } else {
                    let alts: Vec<String> =
                        group.iter().map(|d| d.to_string()).collect();
                    let alts_str = alts.join(" | ");
                    writeln!(f, "  - ({})", alts_str.blue())?;
                }
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
        write!(f, "{} ({})", self.name, self.range)?;
        Ok(())
    }
}

impl Display for PackageVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.name, self.version)?;
        Ok(())
    }
}

impl Default for AuthorAboutData {
    fn default() -> Self {
        AuthorAboutData {
            name: username(),
            email: generate_email_address(),
        }
    }
}

impl Default for PackageAboutData {
    fn default() -> Self {
        PackageAboutData {
            name: "default-package".to_string(),
            version: Version::default(),
            description: String::default(), // Default to empty string
        }
    }
}

impl Default for PackageRange {
    fn default() -> Self {
        PackageRange {
            name: "default-dependency".to_string(),
            range: VersionRange::default(),
        }
    }
}

impl Default for PackageVersion {
    fn default() -> Self {
        PackageVersion {
            name: "default-version".to_string(),
            version: Version::default(),
        }
    }
}

impl RelationData {
    fn is_empty(&self) -> bool {
        self.depend.is_empty()
            && self.depend_cmds.is_empty()
            && self.suggests.is_empty()
            && self.recommends.is_empty()
            && self.conflicts.is_empty()
            && self.virtuals.is_empty()
            && self.provide_cmds.is_empty()
    }
}

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
            description: "This is a test package for demonstration."
                .to_string(), // Added description
        };

        data.architecture =
            vec!["x86_64".to_string(), "aarch64".to_string()];
        data.mode = Mode::Global;

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

        data.relation.suggests.push(vec![PackageRange {
            name: "suggest-x".to_string(),
            range: VersionRange::from_str("= 3.0").unwrap(),
        }]);

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

        data.relation.conflicts.push(PackageRange {
            name: "old-package".to_string(),
            range: VersionRange::from_str("0.9.0").unwrap(),
        });

        data.relation.virtuals.push(PackageVersion {
            name: "my-virtual-pkg".to_string(),
            version: Version::from_str("1.0.0").unwrap(),
        });

        data.relation.provide_cmds.push("my-command".to_string());
        data.relation.provide_cmds.push("another-command".to_string());
        data.relation.depend_cmds.push("git".to_string());
        data.relation.depend_cmds.push("make".to_string());

        println!("\n--- Test Display With Relations and New Fields ---");
        println!("{}", data);
        assert_eq!(
            data.architecture,
            vec!["x86_64".to_string(), "aarch64".to_string()]
        );
        assert_eq!(data.mode, Mode::Global);
    }

    #[test]
    fn test_display_author() {
        let author = AuthorAboutData {
            name: "Test Author".to_string(),
            email: "test@example.com".to_string(),
        };
        println!("\n--- Test Display Author ---");
        println!("{}", author);
    }

    #[test]
    fn test_display_package() {
        let package = PackageAboutData {
            name: "test-package".to_string(),
            version: Version::default(),
            description: "A short description of the test package."
                .to_string(), // Added description
        };
        println!("\n--- Test Display Package ---");
        println!("{}", package);

        let package_no_desc = PackageAboutData {
            name: "test-package-no-desc".to_string(),
            version: Version::default(),
            description: String::new(), // Empty description
        };
        println!("\n--- Test Display Package (No Description) ---");
        println!("{}", package_no_desc);
    }

    #[test]
    fn test_display_relation() {
        let mut relation = RelationData::default();
        relation.depend.push(vec![PackageRange {
            name: "dep-a".to_string(),
            range: VersionRange::from_str(">= 1.0").unwrap(),
        }]);
        relation.suggests.push(vec![PackageRange {
            name: "suggest-x".to_string(),
            range: VersionRange::from_str("= 3.0").unwrap(),
        }]);
        relation.conflicts.push(PackageRange {
            name: "conflicting-pkg".to_string(),
            range: VersionRange::from_str("< 1.0").unwrap(),
        });
        println!("\n--- Test Display Relation ---");
        println!("{}", relation);
    }

    #[test]
    fn test_display_package_range() {
        let range = PackageRange {
            name: "test-dep".to_string(),
            range: VersionRange::from_str(">= 1.0").unwrap(),
        };
        println!("\n--- Test Display Package Range ---");
        println!("{}", range);
    }

    #[test]
    fn test_display_package_version() {
        let version = PackageVersion {
            name: "test-version".to_string(),
            version: Version::default(),
        };
        println!("\n--- Test Display Package Version ---");
        println!("{}", version);
    }

    #[test]
    fn test_mode_display() {
        println!("\n--- Test Mode Display ---");
        println!("Local: {}", Mode::Local);
        println!("Global: {}", Mode::Global);
        println!("Any: {}", Mode::Any);
    }
}

pub fn pkg(args: Vec<&Option>) -> Result<(), std::io::Error> {
    if args.is_empty() {
        return messages::unknown();
    }

    let sub_cmd = args.first().unwrap().to_owned();
    let sub_args: Vec<&Option> = args[1..].to_vec();
    match sub_cmd.opt_str.as_str() {
        "install" | "-i" | "--install" => install::install(sub_args),
        "remove" | "-r" | "--remove" => remove::remove(sub_args),
        "purge" | "-p" | "--purge" => purge::purge(sub_args),
        "list" | "-l" | "--list" => list::list(sub_args),
        "metadata" | "info" => metadata::metadata(sub_args),
        _ => messages::unknown(),
    }
}
