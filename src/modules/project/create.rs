use std::io;
use std::str::FromStr;
use thiserror::Error;
pub mod templates;
use super::super::pkg::{AuthorAboutData, PackageData};
use crate::utils::color::colorize::*;
use crate::utils::files::file_creation;
use clap;
use std::fmt::{self, Display, Formatter};

#[derive(PartialEq, Eq, Default, clap::ValueEnum, Clone, Copy)]
pub enum ProjectTemplateType {
    #[default]
    Default,
    Rust,
    Python,
    Dotnet,
    CLang,
}
impl fmt::Debug for ProjectTemplateType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}
impl FromStr for ProjectTemplateType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "default" => Ok(Self::Default),
            "rust" => Ok(Self::Rust),
            "python" => Ok(Self::Python),
            "dotnet" => Ok(Self::Dotnet),
            "clang" | "cpp" => Ok(Self::CLang),
            _ => Err(format!("Unavailable Template: '{}'", s)),
        }
    }
}

#[derive(Default)]
pub struct ProjectParams {
    pub project_name: String,
    pub project_template: ProjectTemplateType,
    pub author: AuthorAboutData,
}

impl Display for ProjectParams {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}: {}", "Project".bold(), self.project_name)?;
        writeln!(f, "{}: {}", "Template".bold(), self.project_template)?;
        writeln!(
            f,
            "{}: {} <{}>",
            "Author".bold(),
            self.author.name,
            self.author.email
        )
    }
}
impl Display for ProjectTemplateType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let template_str = match self {
            Self::Default => "default",
            Self::Rust => "rust",
            Self::Python => "python",
            Self::Dotnet => "dotnet",
            Self::CLang => "clang",
        };
        write!(f, "{}", template_str)
    }
}

#[derive(Debug, Error)]
pub enum ProjectCreationError {
    #[error("YAML serialization/deserialization error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("Template creation error: {0}")]
    Template(String),
}

pub fn create(params: &ProjectParams) -> Result<(), ProjectCreationError> {
    let mut project_data = PackageData::default();
    project_data.about.package.name = params.project_name.clone();
    project_data.about.author = params.author.clone();

    let project_data = match params.project_template {
        ProjectTemplateType::Default => templates::default(project_data)
            .map_err(|e| ProjectCreationError::Template(e.to_string())),
        ProjectTemplateType::Rust => templates::rust(project_data)
            .map_err(|e| ProjectCreationError::Template(e.to_string())),
        ProjectTemplateType::Python => templates::python(project_data)
            .map_err(|e| ProjectCreationError::Template(e.to_string())),
        ProjectTemplateType::Dotnet => templates::dotnet(project_data)
            .map_err(|e| ProjectCreationError::Template(e.to_string())),
        ProjectTemplateType::CLang => templates::clang(project_data)
            .map_err(|e| ProjectCreationError::Template(e.to_string())),
    }?;

    let project_data_filename = "ipak/project.yaml";
    let data = serde_yaml::to_string(&project_data)?;

    file_creation(project_data_filename, &data)
        .map_err(ProjectCreationError::Io)?;

    Ok(())
}
