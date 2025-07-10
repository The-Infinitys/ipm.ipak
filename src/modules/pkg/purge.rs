use super::super::pkg;
use super::super::project;
use super::super::project::ExecMode;
use super::depend;
use crate::dprintln;
use crate::modules::system::path;
use crate::utils::error::Error;
use std::env;
use std::path::PathBuf;

pub fn purge(
    target_pkg_name: String,
    uninstall_mode: ExecMode,
) -> Result<(), Error> {
    let final_pkg_destination_path = match uninstall_mode {
        ExecMode::Local => {
            path::local::packages_dirpath().join(&target_pkg_name)
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
                .join(&target_pkg_name)
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
        &target_pkg_name,
        uninstall_mode,
        &final_pkg_destination_path,
    )?;

    
    remove_package_from_list(&target_pkg_name, uninstall_mode)?;

    Ok(())
}

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

fn remove_package_from_list(
    pkg_name: &str,
    uninstall_mode: ExecMode,
) -> Result<(), std::io::Error> {
    match uninstall_mode {
        ExecMode::Local => {
            pkg::list::del_pkg_local(pkg_name)?;
            dprintln!("Purged package '{}' from local list.", pkg_name);
        }
        ExecMode::Global => {
            pkg::list::del_pkg_global(pkg_name)?;
            dprintln!("Purged package '{}' from global list.", pkg_name);
        }
    }
    Ok(())
}

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
            let opts = project::purge::PurgeOptions {
                purge_mode: uninstall_mode,
                purge_shell: project::ExecShell::default(),
            };
            project::purge::purge(opts).map_err(std::io::Error::other)?;
            Ok(())
        }
        Err(e) => {
            eprintln!("You cannot uninstall this package.\n{}", e);
            Err(std::io::Error::new(std::io::ErrorKind::Unsupported, e))
        }
    }
}
