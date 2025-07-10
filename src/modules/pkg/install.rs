//! このモジュールは、`ipak`パッケージのインストールに関連する機能を提供します。
//! パッケージアーカイブの展開、ファイルの配置、パッケージリストの更新などを扱います。

use super::super::pkg;
use super::super::project::ExecMode;
use super::depend;
use crate::dprintln;
use crate::modules::project;
use crate::modules::system::path;
use crate::utils::archive::extract_archive;
use crate::utils::error::Error;
use chrono::Local;
use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use tempfile::tempdir;

/// 指定されたパッケージアーカイブをシステムにインストールします。
///
/// パッケージアーカイブを一時ディレクトリに展開し、指定されたインストールモード（ローカルまたはグローバル）
/// に基づいて適切な場所にファイルを配置します。その後、パッケージリストを更新します。
///
/// # Arguments
/// * `file_path` - インストールするパッケージアーカイブへのパス。
/// * `install_mode` - インストールモード（`ExecMode::Local`または`ExecMode::Global`）。
///
/// # Returns
/// `Ok(())` パッケージが正常にインストールされた場合。
/// `Err(Error)` ファイルが見つからない、アーカイブの展開、ファイルの配置、またはパッケージリストの更新中にエラーが発生した場合。
pub fn install(
    file_path: PathBuf,
    install_mode: ExecMode,
) -> Result<(), Error> {
    let target_path = env::current_dir()?.join(&file_path);

    if !target_path.is_file() {
        eprintln!("Couldn't find target file: {}", target_path.display());
        return Err(Error::from(std::io::ErrorKind::NotFound));
    }

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

    dprintln!(
        "Extracting archive from {} to {}",
        pkg_archive_in_temp.display(),
        temp_dir.path().display()
    );
    extract_archive(&pkg_archive_in_temp, &temp_dir.path().to_path_buf())?;
    fs::remove_file(&pkg_archive_in_temp)?;

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

    fs::create_dir_all(&final_pkg_destination_path)?;
    for entry in fs::read_dir(temp_dir.path())? {
        let entry = entry?;
        let target_path =
            final_pkg_destination_path.join(entry.file_name());
        if entry.path().is_dir() {
            fs::create_dir_all(&target_path)?;
            copy_dir_all(&entry.path(), &target_path)?;
        } else {
            fs::copy(entry.path(), &target_path)?;
        }
    }

    dprintln!(
        "Successfully installed package to {}",
        final_pkg_destination_path.display()
    );

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

/// ディレクトリの内容を再帰的にコピーします。
///
/// # Arguments
/// * `src` - コピー元のディレクトリパス。
/// * `dst` - コピー先のディレクトリパス。
///
/// # Returns
/// `Ok(())` コピーが正常に完了した場合。
/// `Err(std::io::Error)` コピー中にエラーが発生した場合。
fn copy_dir_all(src: &PathBuf, dst: &Path) -> std::io::Result<()> {
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

/// パッケージのインストールプロセスを実行します。
///
/// 依存関係グラフをチェックし、パッケージがインストール可能であれば、
/// プロジェクトのインストールスクリプトを実行し、パッケージのメタデータを返します。
///
/// # Arguments
/// * `install_mode` - インストールモード。
///
/// # Returns
/// `Ok(pkg::PackageData)` インストールされたパッケージのメタデータ。
/// `Err(std::io::Error)` 依存関係の競合、またはインストールスクリプトの実行中にエラーが発生した場合。
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
    let package_data = project::metadata::metadata()?; // Call once
    match depend_graph.is_packages_installable(vec![package_data.clone()]) {
        Ok(()) => {
            let opts = project::install::InstallOptions {
                install_mode,
                install_shell: project::ExecShell::default(),
            };
            project::install::install(opts)
                .map_err(std::io::Error::other)?;
            Ok(package_data)
        }
        Err(e) => {
            eprintln!("You cannot install this package.\n{}", e);
            Err(std::io::Error::new(std::io::ErrorKind::Unsupported, e))
        }
    }
}
