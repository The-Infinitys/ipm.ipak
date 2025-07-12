//! このモジュールは、`ipak`パッケージのメタデータ表示に関連する機能を提供します。
//! パッケージアーカイブからメタデータを抽出し、表示します。

use super::super::pkg;
use crate::modules::pkg::PackageData;
use crate::modules::project;
use crate::utils::archive::extract_archive;
use crate::utils::error::Error;
use std::env;
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

pub fn get(target_path: &PathBuf) -> Result<PackageData, Error> {
    let target_path = env::current_dir()?.join(target_path);

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

    let temp_dir = tempdir()?;
    log::debug!("Created temp directory at {}", temp_dir.path().display());

    let pkg_archive_in_temp = temp_dir.path().join(
        target_path.file_name().ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Target path has no filename",
            )
        })?,
    );

    fs::copy(&target_path, &pkg_archive_in_temp)?;
    log::debug!(
        "Copied package to temp directory: {}",
        pkg_archive_in_temp.display()
    );

    log::debug!(
        "Extracting archive from {} to {}",
        pkg_archive_in_temp.display(),
        temp_dir.path().display()
    );
    extract_archive(&pkg_archive_in_temp, &temp_dir.path().to_path_buf())?;
    fs::remove_file(&pkg_archive_in_temp)?;

    let metadata_process_result = {
        let original_cwd = env::current_dir()?;
        env::set_current_dir(temp_dir.path())?;
        log::debug!(
            "Changed current directory to {}",
            temp_dir.path().display()
        );

        let result = metadata_process();

        env::set_current_dir(&original_cwd)?;
        log::debug!(
            "Restored current directory to {}",
            original_cwd.display()
        );
        result
    };
    metadata_process_result.map_err(Error::from)
}
/// 指定されたパッケージアーカイブからメタデータを抽出し、表示します。
///
/// パッケージアーカイブを一時ディレクトリにコピーし、展開した後、
/// その中の`ipak/project.yaml`からメタデータを読み込み、標準出力に表示します。
///
/// # Arguments
/// * `target_path` - メタデータを取得するパッケージアーカイブへのパス。
///
/// # Returns
/// `Ok(())` メタデータが正常に表示された場合。
/// `Err(Error)` ファイルが見つからない、アーカイブの展開、またはメタデータの読み込みに失敗した場合。
pub fn metadata(target_path: &PathBuf) -> Result<(), Error> {
    let pkg_data = get(target_path)?;
    log::info!("{}", pkg_data);
    Ok(())
}

/// 現在のディレクトリの`ipak`プロジェクトメタデータを読み込みます。
///
/// この関数は、主に一時ディレクトリに展開されたパッケージのメタデータを読み込むために使用されます。
///
/// # Returns
/// `Ok(pkg::PackageData)` 読み込まれたパッケージメタデータ。
/// `Err(std::io::Error)` メタデータの読み込みに失敗した場合。
fn metadata_process() -> Result<pkg::PackageData, std::io::Error> {
    project::metadata::metadata()
}
