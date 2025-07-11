//! このモジュールは、`ipak`パッケージの削除に関連する機能を提供します。
//! パッケージのアンインストール、パッケージリストからの削除、依存関係のチェックなどを扱います。

use super::super::pkg;
use super::super::project;
use super::super::project::ExecMode;
use super::depend;
use crate::dprintln;
use crate::modules::pkg::lock::LockManager;
use crate::modules::system::path;
use crate::utils::error::Error;
use std::env;
use std::path::PathBuf;

/// 指定されたパッケージをシステムから削除します。
///
/// アンインストールモード（ローカルまたはグローバル）に基づいて、パッケージの場所を特定し、
/// アンインストールプロセスを実行し、パッケージリストからエントリを削除します。
///
/// # Arguments
/// * `target_pkg_name` - 削除するパッケージの名前。
/// * `uninstall_mode` - アンインストールモード（`ExecMode::Local`または`ExecMode::Global`）。
///
/// # Returns
/// `Ok(())` パッケージが正常に削除された場合。
/// `Err(Error)` パッケージが見つからない、またはアンインストール中にエラーが発生した場合。
pub fn remove(
    target_pkg_names: &Vec<String>,
    uninstall_mode: ExecMode,
) -> Result<(), Error> {
    let lock_manager = LockManager::new(matches!(uninstall_mode, ExecMode::Global));
    lock_manager.acquire_lock()?;

    for target_pkg_name in target_pkg_names {
        let final_pkg_destination_path = match uninstall_mode {
            ExecMode::Local => {
                path::local::packages_dirpath().join(target_pkg_name)
            }
            ExecMode::Global => {
                let list_file_path = path::global::packageslist_filepath();
                list_file_path
                    .parent()
                    .ok_or_else(|| {
                        std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            format!(
                                "Global packages list file path '{}' does not have a parent directory.",
                                list_file_path.display()
                            ),
                        )
                    })?
                    .join(target_pkg_name)
            }
        };

        if !final_pkg_destination_path.exists() {
            eprintln!(
                "Package not found at: {}",
                final_pkg_destination_path.display()
            );
            return Err(std::io::ErrorKind::NotFound.into());
        }

        uninstall_package(
            target_pkg_name,
            uninstall_mode,
            &final_pkg_destination_path,
        )?;

        remove_package_from_list(target_pkg_name, uninstall_mode)?;
    }

    lock_manager.release_lock()?;

    Ok(())
}

/// パッケージのアンインストールプロセスを実行します。
///
/// 指定されたパッケージのディレクトリに移動し、アンインストールスクリプトを実行します。
/// 実行後、元の作業ディレクトリに戻ります。
///
/// # Arguments
/// * `pkg_name` - アンインストールするパッケージの名前。
/// * `uninstall_mode` - アンインストールモード。
/// * `final_pkg_destination_path` - パッケージがインストールされているパス。
///
/// # Returns
/// `Ok(())` アンインストールプロセスが正常に完了した場合。
/// `Err(std::io::Error)` ディレクトリの変更、またはアンインストールスクリプトの実行中にエラーが発生した場合。
fn uninstall_package(
    pkg_name: &str,
    uninstall_mode: ExecMode,
    final_pkg_destination_path: &PathBuf,
) -> Result<(), std::io::Error> {
    let original_cwd = env::current_dir()?;

    if !final_pkg_destination_path.is_dir() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!(
                "Target package directory not found: {}. Expected after extracting {}.",
                final_pkg_destination_path.display(),
                pkg_name
            ),
        ));
    }

    env::set_current_dir(final_pkg_destination_path)?;
    dprintln!(
        "Changed current directory to {}",
        final_pkg_destination_path.display()
    );

    let result = uninstall_process(pkg_name, uninstall_mode);

    env::set_current_dir(&original_cwd)?;
    dprintln!("Restored current directory to {}", original_cwd.display());

    result
}

/// パッケージをローカルまたはグローバルリストから削除します。
///
/// # Arguments
/// * `pkg_name` - 削除するパッケージの名前。
/// * `uninstall_mode` - アンインストールモード。
///
/// # Returns
/// `Ok(())` パッケージがリストから正常に削除された場合。
/// `Err(std::io::Error)` リストからの削除中にエラーが発生した場合。
fn remove_package_from_list(
    pkg_name: &str,
    uninstall_mode: ExecMode,
) -> Result<(), std::io::Error> {
    match uninstall_mode {
        ExecMode::Local => {
            pkg::list::del_pkg_local(pkg_name)?;
            dprintln!("Removed package '{}' from local list.", pkg_name);
        }
        ExecMode::Global => {
            pkg::list::del_pkg_global(pkg_name)?;
            dprintln!("Removed package '{}' from global list.", pkg_name);
        }
    }
    Ok(())
}

/// パッケージのアンインストールプロセスを実行します。
///
/// 依存関係グラフをチェックし、パッケージが削除可能であれば、
/// プロジェクトの削除スクリプトを実行します。
///
/// # Arguments
/// * `pkg_name` - アンインストールするパッケージの名前。
/// * `uninstall_mode` - アンインストールモード。
///
/// # Returns
/// `Ok(())` アンインストールプロセスが正常に完了した場合。
/// `Err(std::io::Error)` 依存関係の競合、または削除スクリプトの実行中にエラーが発生した場合。
fn uninstall_process(
    pkg_name: &str,
    uninstall_mode: ExecMode,
) -> Result<(), std::io::Error> {
    let installed_packages = match uninstall_mode {
        ExecMode::Local => pkg::list::get_local()?,
        ExecMode::Global => pkg::list::get_global()?,
    };

    let depend_graph = depend::DependencyGraph::from_installed_packages(
        &installed_packages,
    );

    match depend_graph.is_packages_removable(&[pkg_name]) {
        Ok(()) => {
            let opts = project::remove::RemoveOptions {
                remove_mode: uninstall_mode,
                remove_shell: project::ExecShell::default(),
            };
            project::remove::remove(opts)
                .map_err(std::io::Error::other)?;
            Ok(())
        }
        Err(e) => {
            eprintln!("You cannot uninstall this package.\n{}", e);
            Err(std::io::Error::new(std::io::ErrorKind::Unsupported, e))
        }
    }
}
