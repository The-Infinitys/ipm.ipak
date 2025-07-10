use super::super::pkg;
use crate::dprintln;
use crate::modules::project;
use crate::utils::archive::extract_archive;
use crate::utils::error::Error;
use std::env;
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir; // 追加
pub fn metadata(target_path: PathBuf) -> Result<(), Error> {
    let target_path = env::current_dir()?.join(&target_path);

    if !target_path.is_file() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!(
                "Couldn't find target file: {}",
                target_path.display()
            ),
        )
        .into());
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

    let metadata_process_result = {
        let original_cwd = env::current_dir()?;
        env::set_current_dir(temp_dir.path())?;
        dprintln!(
            "Changed current directory to {}",
            temp_dir.path().display()
        );

        let result = metadata_process();

        env::set_current_dir(&original_cwd)?;
        dprintln!(
            "Restored current directory to {}",
            original_cwd.display()
        );
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
