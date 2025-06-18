use super::super::pkg;
use super::super::project::ExecMode;
use super::depend;
use crate::dprintln;
use crate::modules::project;
use crate::modules::system::path;
use crate::utils::archive::extract_archive;
use chrono::Local;
use cmd_arg::cmd_arg::{Option, OptionType};
use std::env;
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir; // 追加

pub fn install(args: Vec<&Option>) -> Result<(), std::io::Error> {
    let mut target_path_str = String::new();
    let mut install_mode = ExecMode::default();
    for arg in args {
        match arg.opt_type {
            OptionType::Simple => target_path_str = arg.opt_str.to_owned(),
            OptionType::LongOpt => match arg.opt_str.as_str() {
                "--local" => install_mode = ExecMode::Local,
                "--global" => install_mode = ExecMode::Global,
                _ => continue,
            },
            _ => continue,
        }
    }
    let install_mode = install_mode;
    let target_path = env::current_dir()?.join(&target_path_str);

    if !target_path.is_file() {
        eprintln!("Couldn't find target file: {}", target_path.display());
        return Err(std::io::Error::from(std::io::ErrorKind::NotFound));
    }

    // 一時ディレクトリを作成
    let temp_dir = tempdir()?;
    dprintln!("Created temp directory at {}", temp_dir.path().display());

    let pkg_archive_in_temp = temp_dir.path().join(
        target_path.file_name().ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Target path has no filename",
            )
        })?,
    );

    fs::copy(&target_path, &pkg_archive_in_temp)?;
    dprintln!(
        "Copied package to temp directory: {}",
        pkg_archive_in_temp.display()
    );

    // アーカイブを展開
    dprintln!(
        "Extracting archive from {} to {}",
        pkg_archive_in_temp.display(),
        temp_dir.path().display()
    );
    extract_archive(&pkg_archive_in_temp, &temp_dir.path().to_path_buf())?;
    fs::remove_file(&pkg_archive_in_temp)?;

    // パッケージデータの処理
    let install_process_result = {
        let original_cwd = env::current_dir()?;
        env::set_current_dir(temp_dir.path())?;
        dprintln!(
            "Changed current directory to {}",
            temp_dir.path().display()
        );

        let result = installation_process(install_mode);

        env::set_current_dir(&original_cwd)?;
        dprintln!(
            "Restored current directory to {}",
            original_cwd.display()
        );
        result
    };
    let pkg_data = install_process_result?;

    let final_destination_base_dir: PathBuf = match install_mode {
        ExecMode::Local => path::local::packages_dirpath(),
        ExecMode::Global => {
            let list_file_path = path::global::packageslist_filepath();
            list_file_path.parent().ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!(
                        "Global packages list file path '{}' does not have a parent directory.",
                        list_file_path.display()
                    ),
                )
            })?.to_path_buf()
        }
    };

    fs::create_dir_all(&final_destination_base_dir)?;
    let pkg_name = pkg_data.about.package.name.clone();
    let final_pkg_destination_path =
        final_destination_base_dir.join(&pkg_name);

    if final_pkg_destination_path.exists() {
        if final_pkg_destination_path.is_dir() {
            fs::remove_dir_all(&final_pkg_destination_path)?;
        } else {
            fs::remove_file(&final_pkg_destination_path)?;
        }
    }

    // 展開されたファイルを直接コピー
    fs::create_dir_all(&final_pkg_destination_path)?;
    for entry in fs::read_dir(temp_dir.path())? {
        let entry = entry?;
        let target_path =
            final_pkg_destination_path.join(entry.file_name());
        if entry.path().is_dir() {
            fs::create_dir_all(&target_path)?;
            copy_dir_all(&entry.path(), &target_path)?;
        } else {
            fs::copy(&entry.path(), &target_path)?;
        }
    }

    dprintln!(
        "Successfully installed package to {}",
        final_pkg_destination_path.display()
    );

    // インストール済みパッケージ情報をリストに追加
    let installed_package_data = pkg::list::InstalledPackageData {
        info: pkg_data,
        last_modified: Local::now(),
    };

    match install_mode {
        ExecMode::Local => {
            pkg::list::add_pkg_local(installed_package_data)?;
            dprintln!("Added package '{}' to local list.", pkg_name);
        }
        ExecMode::Global => {
            pkg::list::add_pkg_global(installed_package_data)?;
            dprintln!("Added package '{}' to global list.", pkg_name);
        }
    }

    Ok(())
}

// ディレクトリ全体をコピーするヘルパー関数
fn copy_dir_all(src: &PathBuf, dst: &PathBuf) -> std::io::Result<()> {
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let target = dst.join(entry.file_name());

        if ty.is_dir() {
            fs::create_dir_all(&target)?;
            copy_dir_all(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}

fn installation_process(
    install_mode: ExecMode,
) -> Result<pkg::PackageData, std::io::Error> {
    let installed_packages = match install_mode {
        ExecMode::Local => pkg::list::get_local()?,
        ExecMode::Global => pkg::list::get_global()?,
    };
    let depend_graph = depend::DependencyGraph::from_installed_packages(
        &installed_packages,
    );
    let package_data = project::metadata::metadata()?;
    match depend_graph.is_packages_installable(vec![package_data]) {
        Ok(()) => {
            let opts = project::install::InstallOptions {
                install_mode,
                install_shell: project::ExecShell::default(),
            };
            project::install::install(opts)
                .map_err(std::io::Error::other)?;
            Ok(project::metadata::metadata()?)
        }
        Err(e) => {
            eprintln!("You cannot install this package.\n{}", e);
            Err(std::io::Error::new(std::io::ErrorKind::Unsupported, e))
        }
    }
}
