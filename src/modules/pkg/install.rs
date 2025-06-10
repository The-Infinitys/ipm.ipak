use super::super::pkg;
use super::super::project::ExecMode;
use super::depend;
use crate::dprintln;
use crate::modules::project;
use crate::modules::system::path;
use crate::utils::archive::extract_archive; // archive.rsからインポート
use chrono::Local;
use cmd_arg::cmd_arg::{Option, OptionType};
use std::env;
use std::fs;
use std::path::PathBuf;

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

    let cache_path = path::local::cache_path();

    if cache_path.is_file() {
        fs::remove_file(&cache_path)?;
    }
    if cache_path.is_dir() && cache_path.read_dir()?.next().is_some() {
        fs::remove_dir_all(&cache_path)?;
    }
    if !cache_path.is_dir() {
        fs::create_dir_all(&cache_path)?;
    }

    let pkg_archive_in_cache =
        cache_path.join(target_path.file_name().ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Target path has no filename",
            )
        })?);
    fs::copy(&target_path, &pkg_archive_in_cache)?;

    dprintln!(
        "Created cache for {} at {}",
        target_path.display(),
        pkg_archive_in_cache.display()
    );

    // アーカイブを展開（tarコマンドの代わりにextract_archiveを使用）
    dprintln!(
        "Trying to extract: from {}, to {}",
        pkg_archive_in_cache.display(),
        cache_path.display()
    );
    extract_archive(pkg_archive_in_cache.clone(), cache_path.clone())?;

    fs::remove_file(&pkg_archive_in_cache)?;

    let pkg_filename_str = pkg_archive_in_cache
        .file_name()
        .ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Invalid package archive path (no filename component)",
            )
        })?
        .to_str()
        .ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Invalid package file name (non-UTF8)",
            )
        })?;

    let parts: Vec<&str> = pkg_filename_str.split('.').collect();
    let extracted_dir_name = if parts.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Package filename resulted in empty parts",
        ));
    } else if parts.len() == 1 {
        parts[0].to_string()
    } else {
        parts[0..parts.len() - 1].join(".")
    };

    let extracted_pkg_dir_in_cache = cache_path.join(&extracted_dir_name);

    let install_process_result = {
        let original_cwd = env::current_dir()?;
        if !extracted_pkg_dir_in_cache.is_dir() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!(
                    "Extracted package directory not found: {}. Expected after extracting {}.",
                    extracted_pkg_dir_in_cache.display(),
                    pkg_filename_str
                ),
            ));
        }
        env::set_current_dir(&extracted_pkg_dir_in_cache)?;
        dprintln!(
            "Changed current directory to {}",
            extracted_pkg_dir_in_cache.display()
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

    let source_dir_to_move = extracted_pkg_dir_in_cache;

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
    dprintln!(
        "Ensured final destination base directory exists: {}",
        final_destination_base_dir.display()
    );
    let pkg_name = pkg_data.about.package.name.clone();
    let final_pkg_destination_path =
        final_destination_base_dir.join(&pkg_name);

    if final_pkg_destination_path.exists() {
        dprintln!(
            "Removing existing item at final destination: {}",
            final_pkg_destination_path.display()
        );
        if final_pkg_destination_path.is_dir() {
            fs::remove_dir_all(&final_pkg_destination_path)?;
        } else {
            fs::remove_file(&final_pkg_destination_path)?;
        }
    }

    fs::rename(&source_dir_to_move, &final_pkg_destination_path)?;
    dprintln!(
        "Successfully moved package from {} to {}",
        source_dir_to_move.display(),
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
