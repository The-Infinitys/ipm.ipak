use crate::utils::{
    self,
    shell::{self, question},
};
use build::BuildOptions;
use cmd_arg::cmd_arg::Option;
use install::InstallOptions; // Import InstallOptions
use purge::PurgeOptions; // Import PurgeOptions
use remove::RemoveOptions; // Import RemoveOptions
use std::{env, fs, str::FromStr};
mod build;
mod create;
mod init;
pub mod install;
pub mod metadata;
mod package;
pub mod purge;
pub mod remove;
pub mod run;
use super::pkg::AuthorAboutData;
use create::{ProjectParams, ProjectTemplateType};
use std::fmt::{self, Display};
use std::process::Command;
#[derive(PartialEq, Eq, Clone, Copy)]
pub enum ExecMode {
    Local,
    Global,
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

#[derive(Default)]
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
        // `pub` を追加して外部からも利用可能に
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

/// 指定されたオプションからシェルタイプをパースするヘルパー関数
fn parse_shell_option(arg: &Option) -> Result<ExecShell, String> {
    arg.opt_values
        .first()
        .ok_or_else(|| "Missing value for shell option".to_string())? // エラーメッセージを改善
        .parse::<ExecShell>() // FromStr for ExecShell を利用
}

/// プロジェクト関連のサブコマンドを処理します。
pub fn project(args: Vec<&Option>) -> Result<(), std::io::Error> {
    let sub_cmd = args.first().ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "No subcommand provided",
        )
    })?;

    let sub_args = &args[1..]; // スライスとして渡すことで不必要なコピーを避ける

    match sub_cmd.opt_str.as_str() {
        "create" | "new" => project_create(sub_args.to_vec()), // createはVec<&Option>を要求
        "info" | "metadata" => project_metadata(),
        "build" | "compile" => project_build(sub_args.to_vec()),
        "install" => project_install(sub_args.to_vec()),
        "remove" => project_remove(sub_args.to_vec()),
        "purge" => project_purge(sub_args.to_vec()),
        "package" | "pkg" => project_package(sub_args.to_vec()),
        "init" => project_init(),
        "run" => project_run(sub_args.to_vec()),
        _ => project_run(args),
    }
}
fn project_run(args: Vec<&Option>) -> Result<(), std::io::Error> {
    let sub_cmd = args
        .first()
        .ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "No subcommand provided",
            )
        })?
        .opt_str
        .clone();

    let sub_args = &args[1..]; // スライスとして渡すことで不必要なコピーを避ける
    run::run(&sub_cmd, sub_args.to_vec()).map_err(|e| {
        std::io::Error::other(e.to_string())
    })
}
fn project_init() -> Result<(), std::io::Error> {
    init::init()
}
fn project_package(args: Vec<&Option>) -> Result<(), std::io::Error> {
    let mut package_options = package::PackageOptions::default();

    for arg in args {
        match arg.opt_str.as_str() {
            "--target" | "target" => {
                package_options.target = arg
                    .opt_values
                    .first()
                    .ok_or_else(|| {
                        std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            "Missing value for --target",
                        )
                    })?
                    .parse::<package::PackageTarget>()
                    .map_err(|e| {
                        std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            format!("Error parsing package target: {}", e),
                        )
                    })?;
            }
            _ => {
                eprintln!("Invalid Option: {}", arg.opt_str);
                eprintln!("Available Options: --target");
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!(
                        "Unknown option for package command: {}",
                        arg.opt_str
                    ),
                ));
            }
        }
    }
    package::package(package_options).map_err(std::io::Error::other)
}

fn project_build(args: Vec<&Option>) -> Result<(), std::io::Error> {
    let mut build_options = BuildOptions::default();

    for arg in args {
        match arg.opt_str.as_str() {
            "--release" => {
                build_options.build_mode = build::BuildMode::Release;
            }
            "--debug" => {
                build_options.build_mode = build::BuildMode::Debug;
            }
            "--shell" | "--sh" => {
                build_options.build_shell = parse_shell_option(arg)
                    .map_err(|e| {
                        std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            format!("Error parsing shell option: {}", e),
                        )
                    })?;
            }
            _ => {
                eprintln!("Unknown Option: {}", arg.opt_str);
                eprintln!(
                    "Available Options: --release, --debug, --shell|--sh"
                );
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!(
                        "Unknown option for build command: {}",
                        arg.opt_str
                    ),
                ));
            }
        }
    }
    build::build(build_options).map_err(std::io::Error::other)
}

fn project_install(args: Vec<&Option>) -> Result<(), std::io::Error> {
    let mut install_options = InstallOptions::default();

    for arg in args {
        match arg.opt_str.as_str() {
            "--shell" | "--sh" => {
                install_options.install_shell = parse_shell_option(arg)
                    .map_err(|e| {
                        std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            format!("Error parsing shell option: {}", e),
                        )
                    })?;
            }
            "--local" => {
                install_options.install_mode = ExecMode::Local;
            }
            "--global" => {
                install_options.install_mode = ExecMode::Global;
            }
            _ => {
                eprintln!("Unknown Option: {}", arg.opt_str);
                eprintln!(
                    "Available Options: --global, --local, --shell|--sh"
                );
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!(
                        "Unknown option for install command: {}",
                        arg.opt_str
                    ),
                ));
            }
        }
    }
    install::install(install_options).map_err(std::io::Error::other)
}

fn project_remove(args: Vec<&Option>) -> Result<(), std::io::Error> {
    let mut remove_options = RemoveOptions::default();

    for arg in args {
        match arg.opt_str.as_str() {
            "--shell" | "--sh" => {
                remove_options.remove_shell = parse_shell_option(arg)
                    .map_err(|e| {
                        std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            format!("Error parsing shell option: {}", e),
                        )
                    })?;
            }
            "--local" => {
                remove_options.remove_mode = ExecMode::Local;
            }
            "--global" => {
                remove_options.remove_mode = ExecMode::Global;
            }
            _ => {
                eprintln!("Unknown Option: {}", arg.opt_str);
                eprintln!("Available Options: --shell|--sh");
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!(
                        "Unknown option for remove command: {}",
                        arg.opt_str
                    ),
                ));
            }
        }
    }
    remove::remove(remove_options).map_err(std::io::Error::other)
}

fn project_purge(args: Vec<&Option>) -> Result<(), std::io::Error> {
    let mut purge_options = PurgeOptions::default();

    for arg in args {
        match arg.opt_str.as_str() {
            "--shell" | "--sh" => {
                purge_options.purge_shell = parse_shell_option(arg)
                    .map_err(|e| {
                        std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            format!("Error parsing shell option: {}", e),
                        )
                    })?;
            }
            "--local" => {
                purge_options.purge_mode = ExecMode::Local;
            }
            "--global" => {
                purge_options.purge_mode = ExecMode::Global;
            }
            _ => {
                eprintln!("Unknown Option: {}", arg.opt_str);
                eprintln!("Available Options: --shell|--sh");
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!(
                        "Unknown option for purge command: {}",
                        arg.opt_str
                    ),
                ));
            }
        }
    }
    purge::purge(purge_options).map_err(std::io::Error::other)
}

fn project_metadata() -> Result<(), std::io::Error> {
    metadata::show_metadata().map_err(|_| {
        // エラー詳細が不要な場合は `_` で無視
        std::io::Error::other("failed to get metadata")
    })
}

fn project_create(args: Vec<&Option>) -> Result<(), std::io::Error> {
    let mut params = ProjectParams {
        project_name: String::new(),
        project_template: ProjectTemplateType::Default,
        author: AuthorAboutData {
            name: String::new(),
            email: String::new(),
        },
    };
    let mut _is_template_selected = false;
    for arg in args {
        match arg.opt_str.as_str() {
            "--project-name" | "--name" | "--package-name" => {
                params.project_name = arg
                    .opt_values
                    .first()
                    .ok_or_else(|| {
                        std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            "Missing value for project name",
                        )
                    })?
                    .to_owned();
            }
            "--template" => {
                params.project_template = arg
                    .opt_values
                    .first()
                    .ok_or_else(|| {
                        std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            "Missing value for template",
                        )
                    })?
                    .parse::<ProjectTemplateType>()
                    .map_err(|e| {
                        std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            format!(
                                "Error parsing project template: {}",
                                e
                            ),
                        )
                    })?;
                _is_template_selected = true;
            }
            "--author-name" => {
                params.author.name = arg
                    .opt_values
                    .first()
                    .ok_or_else(|| {
                        std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            "Missing value for author name",
                        )
                    })?
                    .to_owned();
            }
            "--author-email" => {
                params.author.email = arg
                    .opt_values
                    .first()
                    .ok_or_else(|| {
                        std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            "Missing value for author email",
                        )
                    })?
                    .to_owned();
            }
            _ => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!(
                        "Unknown option for create command: {}",
                        arg.opt_str
                    ),
                ));
            }
        }
    }

    if params.project_name.is_empty() {
        params.project_name = question::kebab_loop("Project name");
    }
    if !_is_template_selected {
        params.project_template =
            ProjectTemplateType::from_str(&shell::question::select(
                "Project template",
                &[
                    &format!("{}", ProjectTemplateType::Default),
                    &format!("{}", ProjectTemplateType::Rust),
                    &format!("{}", ProjectTemplateType::Python),
                    &format!("{}", ProjectTemplateType::Dotnet),
                ],
            ))
            .unwrap_or_default();
    }
    if params.author.name.is_empty() {
        params.author.name = shell::username();
    }
    if params.author.email.is_empty() {
        params.author.email = utils::generate_email_address();
    }

    println!("{}", params); // Debugging/User info

    fs::create_dir(&params.project_name).map_err(|err| {
        std::io::Error::other(format!(
            "failed to create dir: {}\nDue to: {}",
            &params.project_name,
            err.kind()
        ))
    })?;

    env::set_current_dir(&params.project_name).map_err(|_| {
        std::io::Error::other(format!(
            "failed to set current dir: {}",
            &params.project_name
        ))
    })?;

    create::create(&params).map_err(|_| {
        std::io::Error::other(format!(
            "failed to create project: {}",
            &params.project_name
        ))
    })?;

    Ok(())
}
