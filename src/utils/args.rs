use crate::modules::project::ProjectTemplateType;
use crate::modules::project::package::PackageTarget;
use crate::{modules::project::ExecShell, utils::archive::ArchiveType};
use clap::{Parser, Subcommand};
use std::path::PathBuf; // Import ExecShell

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(name = "ipak")] // Assuming the name of your application is 'ipak'
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Display version information.
    Version,
    /// Display the manual.
    Manual,
    /// Manage projects.
    #[command(subcommand)]
    Project(ProjectCommands),
    /// Configure system settings.
    #[command(subcommand)]
    System(SystemCommands),
    /// Utility commands.
    #[command(subcommand)]
    Utils(UtilsCommands),
    /// Manage packages.
    #[command(subcommand)]
    Pkg(PkgCommands),
}

#[derive(Subcommand, Debug)]
pub enum ProjectCommands {
    /// Create a new project.
    Create {
        /// Name of the project.
        #[arg(long = "project-name")]
        project_name: String,
        /// Template to use (default or rust).
        #[arg(long)]
        template: Option<ProjectTemplateType>,
        /// Author name for the project.
        #[arg(long)]
        author_name: Option<String>,
        /// Author email for the project.
        #[arg(long)]
        author_email: Option<String>,
    },
    /// Build the project.
    Build {
        /// Build in release mode.
        #[arg(long)]
        release: bool,
        /// Shell to use (bash or zsh).
        #[arg(long)]
        shell: Option<ExecShell>, // Changed to ExecShell
    },
    /// Install the project.
    Install {
        /// Install globally.
        #[arg(long)]
        global: bool,
        /// Shell to use (bash or zsh).
        #[arg(long)]
        shell: Option<ExecShell>, // Changed to ExecShell
    },
    /// Remove the project.
    Remove {
        /// Remove locally.
        #[arg(long)]
        local: bool,
        /// Remove globally.
        #[arg(long)]
        global: bool,
        /// Shell to use (bash or zsh).
        #[arg(long)]
        shell: Option<ExecShell>, // Changed to ExecShell
    },
    /// Completely remove the project and associated data.
    Purge {
        /// Purge locally.
        #[arg(long)]
        local: bool,
        /// Purge globally.
        #[arg(long)]
        global: bool,
        /// Shell to use (bash or zsh).
        #[arg(long)]
        shell: Option<ExecShell>, // Changed to ExecShell
    },
    /// Package the project.
    Package {
        /// Target for packaging (e.g., "tar", "zip").
        #[arg(long)]
        target: Option<PackageTarget>,
    },
    /// Display project metadata.
    Metadata,
    /// Init project.
    Init,
    /// Run an arbitrary command within the project.
    Run {
        /// Shell to run,
        shell: Option<ExecShell>,
        /// Command to run.
        command: String,
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
}

#[derive(Subcommand, Debug)]
pub enum SystemCommands {
    /// Configure local or global settings.
    Configure {
        /// Configure local settings.
        #[arg(long)]
        local: bool,
        /// Configure global settings.
        #[arg(long)]
        global: bool,
    },
}

#[derive(Subcommand, Debug)]
pub enum UtilsCommands {
    /// Archive utilities.
    #[command(subcommand)]
    Archive(ArchiveCommands),
}

#[derive(Subcommand, Debug)]
pub enum ArchiveCommands {
    /// Create an archive.
    Create {
        /// Path to the target to archive.
        #[arg(long)]
        from: PathBuf,
        /// Path to create the archive.
        #[arg(long)]
        to: PathBuf,
        /// Type of archive (tar, zstd, ...).
        #[arg(long)]
        archive_type: ArchiveType,
    },
    /// Extract an archive.
    Extract {
        /// Path to the archive to extract.
        #[arg(long)]
        from: PathBuf,
        /// Path to extract the archive to.
        #[arg(long)]
        to: PathBuf,
    },
}

#[derive(Subcommand, Debug)]
pub enum PkgCommands {
    /// List installed packages.
    List {
        /// List local packages.
        #[arg(long)]
        local: bool,
        /// List global packages.
        #[arg(long)]
        global: bool,
    },
    /// Install a package.
    Install {
        /// Path to the package file.
        #[arg()]
        file_path: PathBuf,
        /// Install locally.
        #[arg(long)]
        local: bool,
        /// Install globally.
        #[arg(long)]
        global: bool,
    },
    /// Remove a package (only binaries are removed, configuration files remain).
    Remove {
        /// Name of the package to remove.
        #[arg()]
        package_name: String,
        /// Remove locally.
        #[arg(long)]
        local: bool,
        /// Remove globally.
        #[arg(long)]
        global: bool,
    },
    /// Purge a package (completely removed, including configuration files).
    Purge {
        /// Name of the package to purge.
        #[arg()]
        package_name: String,
        /// Purge locally.
        #[arg(long)]
        local: bool,
        /// Purge globally.
        #[arg(long)]
        global: bool,
    },
    MetaData {
        /// Nmae of the package to get metadata.
        #[arg()]
        package_path: PathBuf,
    },
}
