use super::super::pkg;
use crate::dprintln;
use crate::modules::project;
use crate::utils::archive::extract_archive;
use cmd_arg::cmd_arg::{Option, OptionType};
use std::env;
use std::fs;
use tempfile::tempdir; // 追加

pub fn metadata(args: Vec<&Option>) -> Result<(), std::io::Error> {
    let mut target_path_str = String::new();
    for arg in args {
        match arg.opt_type {
            OptionType::Simple => target_path_str = arg.opt_str.to_owned(),
            _ => continue,
        }
    }
    let target_path = env::current_dir()?.join(&target_path_str);

    if !target_path.is_file() {
        eprintln!("Couldn't find target file: {}", target_path.display());
        return Err(std::io::Error::from(std::io::ErrorKind::NotFound));
    }

    // 一時ディレクトリを作成
    let temp_dir = tempdir()?;
    dprintln!("Created temp directory at {}", temp_dir.path().display());

    let pkg_archive_in_temp = temp_dir.path().join(
        target_path
            .file_name()
            .ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "Target path has no filename",
                )
            })?
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

    let metadata_process_result = {
        let original_cwd = env::current_dir()?;
        env::set_current_dir(temp_dir.path())?;
        dprintln!(
            "Changed current directory to {}",
            temp_dir.path().display()
        );

        let result = metadata_process();

        env::set_current_dir(&original_cwd)?;
        dprintln!("Restored current directory to {}", original_cwd.display());
        result
    };
    let pkg_data = metadata_process_result?;
    println!("{}", pkg_data);
    Ok(())
}

fn metadata_process() -> Result<pkg::PackageData, std::io::Error> {
    project::metadata::metadata().map_err(std::io::Error::other)?;
    project::metadata::metadata()
}
