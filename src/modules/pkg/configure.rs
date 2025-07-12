//! このモジュールは、`ipak`パッケージの設定に関連する機能を提供します。

use crate::modules::project::ExecMode;
use crate::modules::project::configure as project_configure;
use crate::modules::system::path;
use crate::utils::error::Error;
use std::env;

/// 指定されたパッケージを設定します。
///
/// # Arguments
/// * `package_names` - 設定するパッケージの名前のベクター。
/// * `configure_mode` - 設定モード（`ExecMode::Local`または`ExecMode::Global`）。
///
/// # Returns
/// `Ok(())` パッケージが正常に設定された場合。
/// `Err(Error)` パッケージが見つからない、または設定中にエラーが発生した場合。
pub fn configure(
    package_names: &Vec<String>,
    configure_mode: ExecMode,
) -> Result<(), Error> {
    for package_name in package_names {
        use super::list;
        let installed_packages = match configure_mode {
            ExecMode::Local => list::get_local()?,
            ExecMode::Global => list::get_global()?,
        };

        let _ = installed_packages
            .installed_packages
            .iter()
            .find(|pkgdata| {
                &pkgdata.info.about.package.name == package_name
            })
            .ok_or_else(|| {
                Error::from(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("Package '{}' not found.", package_name),
                ))
            })?; // パッケージが存在するか確認する

        let final_pkg_destination_path = match configure_mode {
            ExecMode::Local => {
                path::local::packages_dirpath().join(package_name)
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
                    .join(package_name)
            }
        };

        if !final_pkg_destination_path.exists() {
            log::error!(
                "Package directory not found at: {}",
                final_pkg_destination_path.display()
            );
            return Err(std::io::ErrorKind::NotFound.into());
        }

        let original_cwd = env::current_dir()?;
        env::set_current_dir(&final_pkg_destination_path)?;
        log::debug!(
            "Changed current directory to {}",
            final_pkg_destination_path.display()
        );
        use super::super::project;
        let opts = project_configure::ConfigureOptions {
            configure_mode,
            configure_shell: project::ExecShell::default(),
        };
        project_configure::configure(opts)
            .map_err(std::io::Error::other)?;

        env::set_current_dir(&original_cwd)?;
        log::debug!(
            "Restored current directory to {}",
            original_cwd.display()
        );

        log::debug!("Successfully configured package: {}", package_name);
    }
    Ok(())
}
