use std::fmt;

#[derive(Debug)]
pub enum InstallError {
    MissingDependencies {
        package: String,
        missing: Vec<Vec<super::PackageRange>>,
    },
    ConflictsWithInstalled {
        package: String,
        conflicts: Vec<super::PackageRange>,
    },
    ConflictsWithOtherPackages {
        package: String,
        conflicts_with: String,
    },
    MissingSystemCommands {
        package: String,
        missing_cmds: Vec<String>,
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
        }
    }
}

impl std::error::Error for InstallError {}

#[derive(Debug)]
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

impl std::error::Error for RemoveError {}