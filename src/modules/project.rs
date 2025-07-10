use crate::modules::project::build::BuildMode;
use crate::modules::project::package::PackageTarget;
use crate::utils::error::Error;
use crate::utils::shell::is_superuser;
use crate::utils::{
    args::ProjectCommands,
    generate_email_address,
    shell::{self, username},
};
use build::BuildOptions;
use install::InstallOptions; 
use purge::PurgeOptions; 
use remove::RemoveOptions; 
use std::{env, fs, str::FromStr};
mod build;
mod create;
mod init;
pub mod install;
pub mod metadata;
pub mod package;
pub mod purge;
pub mod remove;
pub mod run;
use super::pkg::AuthorAboutData;
use clap;
use create::ProjectParams;
pub use create::ProjectTemplateType;
use std::fmt::{self, Display};
use std::process::Command;
#[derive(PartialEq, Eq, Clone, Copy)]
pub enum ExecMode {
    Local,
    Global,
}
impl From<(bool, bool)> for ExecMode {
    
    fn from(value: (bool, bool)) -> Self {
        let (local, global) = value;
        if local && !global {
            Some(true)
        } else if global && !local {
            Some(false)
        } else {
            None
        }
        .into()
    }
}
impl From<Option<bool>> for ExecMode {
    fn from(value: Option<bool>) -> Self {
        match value {
            Some(is_local) => {
                if is_local {
                    ExecMode::Local
                } else {
                    ExecMode::Global
                }
            }
            None => {
                if is_superuser() {
                    ExecMode::Global
                } else {
                    ExecMode::Local
                }
            }
        }
    }
}
impl Display for ExecMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExecMode::Local => write!(f, "local"),
            ExecMode::Global => {
                write!(f, "global")
            }
        }
    }
}
impl Default for ExecMode {
    fn default() -> Self {
        if shell::is_superuser() { Self::Global } else { Self::Local }
    }
}

#[derive(Default, clap::ValueEnum, Clone, Copy)]
pub enum ExecShell {
    RBash,
    #[default]
    Bash,
    Zsh,
    Csh,
}

impl FromStr for ExecShell {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "bash" => Ok(Self::Bash),
            "zsh" => Ok(Self::Zsh),
            "csh" => Ok(Self::Csh),
            "rbash" => Ok(Self::RBash),
            _ => Err(format!("Unavailable Shell: {}", s)),
        }
    }
}

impl ExecShell {
    pub fn generate(&self) -> Command {
        
        match self {
            Self::RBash => {
                let mut cmd = Command::new("bash");
                cmd.arg("-r");
                cmd
            }
            Self::Bash => Command::new("bash"),
            Self::Zsh => Command::new("zsh"),
            Self::Csh => Command::new("csh"),
        }
    }
}

impl Display for ExecShell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExecShell::RBash => {
                write!(f, "restricted bash")
            }
            ExecShell::Bash => write!(f, "bash"),
            ExecShell::Zsh => write!(f, "zsh"),
            ExecShell::Csh => write!(f, "csh"),
        }
    }
}

impl fmt::Debug for ExecShell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExecShell::RBash => {
                write!(f, "restricted bash")
            }
            ExecShell::Bash => write!(f, "bash"),
            ExecShell::Zsh => write!(f, "zsh"),
            ExecShell::Csh => write!(f, "csh"),
        }
    }
}

pub fn project(args: ProjectCommands) -> Result<(), Error> {
    match args {
        ProjectCommands::Create {
            project_name,
            template,
            author_name,
            author_email,
        } => project_create(
            project_name,
            template,
            author_name,
            author_email,
        ),
        ProjectCommands::Metadata => project_metadata(),
        ProjectCommands::Build { release, shell } => {
            project_build(release, shell)
        }
        ProjectCommands::Install { global, shell } => {
            project_install(global, shell)
        }
        ProjectCommands::Remove { local, global, shell } => {
            project_remove((local, global).into(), shell)
        }
        ProjectCommands::Purge { local, global, shell } => {
            project_purge((local, global).into(), shell)
        }
        ProjectCommands::Package { target } => project_package(target),
        ProjectCommands::Init => project_init(),
        ProjectCommands::Run { shell, command, args } => {
            project_run(shell, command, args)
        }
    }
}
fn project_run(
    shell: Option<ExecShell>,
    command: String,
    args: Vec<String>,
) -> Result<(), Error> {
    run::run(shell, &command, args).map_err(|e| -> Error { e.into() })
}
fn project_init() -> Result<(), Error> {
    init::init().map_err(|e| -> Error { e.into() })
}
fn project_package(target: Option<PackageTarget>) -> Result<(), Error> {
    let package_options =
        package::PackageOptions { target: target.unwrap_or_default() };

    package::package(package_options).map_err(|e| -> Error { e.into() })
}

fn project_build(
    release: bool,
    shell: Option<ExecShell>,
) -> Result<(), Error> {
    let build_options = BuildOptions {
        build_mode: if release {
            BuildMode::Release
        } else {
            BuildMode::Debug
        },
        build_shell: shell.unwrap_or_default(),
    };
    build::build(build_options).map_err(|e| -> Error { e.into() })
}

fn project_install(
    global: bool,
    shell: Option<ExecShell>,
) -> Result<(), Error> {
    let install_options = InstallOptions {
        install_shell: shell.unwrap_or_default(),
        install_mode: if global {
            ExecMode::Global
        } else {
            ExecMode::Local
        },
    };
    install::install(install_options).map_err(|e| -> Error { e.into() })
}

fn project_remove(
    remove_mode: ExecMode,
    shell: Option<ExecShell>,
) -> Result<(), Error> {
    let remove_options = RemoveOptions {
        remove_shell: shell.unwrap_or_default(),
        remove_mode,
    };
    remove::remove(remove_options).map_err(|e| -> Error { e.into() })
}

fn project_purge(
    purge_mode: ExecMode,
    shell: Option<ExecShell>,
) -> Result<(), Error> {
    let purge_options = PurgeOptions {
        purge_shell: shell.unwrap_or_default(),
        purge_mode,
    };
    purge::purge(purge_options).map_err(|e| -> Error { e.into() })
}

fn project_metadata() -> Result<(), Error> {
    metadata::show_metadata().map_err(|e| -> Error { e.into() })
}

fn project_create(
    project_name: String,
    template: Option<ProjectTemplateType>,
    author_name: Option<String>,
    author_email: Option<String>,
) -> Result<(), Error> {
    let params = ProjectParams {
        project_name,
        project_template: template.unwrap_or_default(),
        author: AuthorAboutData {
            name: author_name.unwrap_or_else(username),
            email: author_email
                .unwrap_or_else(generate_email_address),
        },
    };
    println!("{}", params); 

    fs::create_dir(&params.project_name)
        .map_err(|err| -> Error { err.into() })?;

    env::set_current_dir(&params.project_name)
        .map_err(|err| -> Error { err.into() })?;

    create::create(&params)
        .map_err(|_| {
            std::io::Error::other(format!(
                "failed to create project: {}",
                &params.project_name
            ))
        })
        .map_err(|err| -> Error { err.into() })?;

    Ok(())
}
