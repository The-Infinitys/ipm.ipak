use thiserror::Error;
#[derive(Debug, Error)]
pub enum IpakError {
    #[error("IO-Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Command Exited with {0}")]
    CommandExecution(i32),
    #[error("Dependency-Error: {0}")]
    InstallDependency(#[from] InstallError),
    #[error("Dependency-Error: {0}")]
    RemoveDependency(#[from] RemoveError),
    #[error("Project-Error: {0}")]
    ProjectCreation(ProjectCreationError),
    #[error("{0}")]
    Message(String),
    #[error("Yaml-Error: {0}")]
    Yaml(#[from] serde_yaml::Error),
    #[error("Ignore-Bandle-Error: {0}")]
    IgnoreBandle(#[from] ignore::Error),
    #[error("Strip-Prefix-Error: {0}")]
    StripPrefix(#[from] StripPrefixError),
}

use crate::modules::project::create::ProjectCreationError;

use crate::modules::pkg::depend::{InstallError, RemoveError};

use std::path::StripPrefixError;
