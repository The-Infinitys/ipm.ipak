use crate::modules::pkg::PackageRange;
use std::fmt; // PackageRange を使用するために追加
use thiserror;
#[derive(Debug, thiserror::Error)]
pub enum InstallError {
    MissingDependencies {
        package: String,
        missing: Vec<Vec<PackageRange>>,
    },
    ConflictsWithInstalled {
        package: String,
        conflicts: Vec<PackageRange>,
    },
    ConflictsWithOtherPackages {
        package: String,
        conflicts_with: String,
    },
    MissingSystemCommands {
        package: String,
        missing_cmds: Vec<String>,
    },
    CyclicDependencies {
        packages: Vec<String>,
    },
}

impl fmt::Display for InstallError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InstallError::MissingDependencies { package, missing } => {
                write!(
                    f,
                    "Package {} has missing dependencies: {:?}",
                    package, missing
                )
            }
            InstallError::ConflictsWithInstalled {
                package,
                conflicts,
            } => {
                write!(
                    f,
                    "Package {} conflicts with installed packages: {:?}",
                    package, conflicts
                )
            }
            InstallError::ConflictsWithOtherPackages {
                package,
                conflicts_with,
            } => {
                write!(
                    f,
                    "Package {} conflicts with another package: {}",
                    package, conflicts_with
                )
            }
            InstallError::MissingSystemCommands {
                package,
                missing_cmds,
            } => {
                write!(
                    f,
                    "Package {} requires unavailable system commands: {:?}",
                    package, missing_cmds
                )
            }
            InstallError::CyclicDependencies { packages } => {
                write!(
                    f,
                    "Cyclic dependencies detected among packages: {:?}",
                    packages
                )
            }
        }
    }
}


#[derive(Debug,thiserror::Error)]
pub enum RemoveError {
    DependencyOfOtherPackages {
        package: String,
        dependent_packages: Vec<String>,
    },
}

impl fmt::Display for RemoveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RemoveError::DependencyOfOtherPackages {
                package,
                dependent_packages,
            } => {
                write!(
                    f,
                    "Package '{}' cannot be removed because the following packages depend on it: {:?}",
                    package, dependent_packages
                )
            }
        }
    }
}

