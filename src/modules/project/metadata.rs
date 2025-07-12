//! このモジュールは、`ipak`プロジェクトのメタデータ（`project.yaml`）の読み書きと管理を行います。
//! プロジェクトディレクトリの探索、メタデータの取得、表示、保存などの機能を提供します。

use crate::dprintln;
use crate::{modules::pkg::PackageData, utils::files::is_file_exists};
use std::{env, io, path::PathBuf};

/// 現在のディレクトリまたは親ディレクトリから`ipak`プロジェクトのルートディレクトリを探索します。
///
/// `ipak/project.yaml`ファイルが存在する最初のディレクトリをプロジェクトのルートと見なします。
///
/// # Returns
/// `Ok(PathBuf)` プロジェクトのルートディレクトリへのパス。
/// `Err(io::Error)` `project.yaml`が見つからない場合、またはパスが無効な場合。
pub fn get_dir() -> Result<PathBuf, io::Error> {
    let mut current_path = env::current_dir()?;
    loop {
        let metadata_path = current_path.join("ipak/project.yaml");
        log::debug!("{}", metadata_path.display());
        if is_file_exists(metadata_path.to_str().ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "Invalid path characters",
            )
        })?) {
            return Ok(current_path);
        } else {
            log::debug!(
                "Not found project.yaml in {}",
                current_path.display()
            );
            if let Some(parent) = current_path.parent() {
                current_path = parent.to_owned();
            } else {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    "project.yaml not found in current or parent directories",
                ));
            }
        }
    }
}

/// `ipak`プロジェクトのメタデータファイル（`project.yaml`）へのパスを返します。
///
/// `get_dir`を使用してプロジェクトのルートディレクトリを特定し、その中の`ipak/project.yaml`へのパスを構築します。
///
/// # Returns
/// `Ok(PathBuf)` メタデータファイルへのパス。
/// `Err(io::Error)` プロジェクトのルートディレクトリが見つからない場合。
pub fn get_path() -> Result<PathBuf, io::Error> {
    get_dir().map(|dir| dir.join("ipak/project.yaml"))
}

/// `ipak`プロジェクトのメタデータを読み込み、`PackageData`構造体として返します。
///
/// `get_path`を使用してメタデータファイルのパスを特定し、その内容をYAMLとしてパースします。
///
/// # Returns
/// `Ok(PackageData)` パースされたプロジェクトメタデータ。
/// `Err(io::Error)` ファイルの読み込みまたはパースに失敗した場合。
pub fn metadata() -> Result<PackageData, io::Error> {
    let metadata_path = get_path()?;
    let read_data =
        std::fs::read_to_string(&metadata_path).map_err(|e| {
            io::Error::new(
                e.kind(),
                format!(
                    "Failed to read {}: {}",
                    metadata_path.display(),
                    e
                ),
            )
        })?;

    serde_yaml::from_str::<PackageData>(&read_data).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Failed to parse {}: {}", metadata_path.display(), e),
        )
    })
}

/// 現在のプロジェクトのメタデータを標準出力に表示します。
///
/// `from_current`を使用してメタデータを読み込み、その`Display`実装を利用して出力します。
///
/// # Returns
/// `Ok(())` 成功した場合。
/// `Err(io::Error)` メタデータの読み込みに失敗した場合。
pub fn show_metadata() -> Result<(), io::Error> {
    let package_data = from_current()?;
    println!("{}", package_data);
    Ok(())
}

/// 指定された`PackageData`を`ipak`プロジェクトのメタデータファイルに書き込みます。
///
/// `get_path`を使用してメタデータファイルのパスを特定し、`PackageData`をYAMLとしてシリアライズして書き込みます。
/// 必要な親ディレクトリが存在しない場合は作成します。
///
/// # Arguments
/// * `package_data` - 書き込む`PackageData`構造体への参照。
///
/// # Returns
/// `Ok(())` 成功した場合。
/// `Err(io::Error)` ファイルの書き込みまたはシリアライズに失敗した場合。
pub fn write(package_data: &PackageData) -> Result<(), io::Error> {
    let metadata_path = get_path()?;

    let parent_dir = metadata_path.parent().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "Could not determine parent directory for project.yaml",
        )
    })?;
    std::fs::create_dir_all(parent_dir)?;

    let yaml_string =
        serde_yaml::to_string(package_data).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Failed to serialize PackageData to YAML: {}", e),
            )
        })?;

    std::fs::write(&metadata_path, yaml_string).map_err(|e| {
        io::Error::new(
            e.kind(),
            format!(
                "Failed to write to {}: {}",
                metadata_path.display(),
                e
            ),
        )
    })?;

    log::debug!(
        "Successfully wrote project metadata to {}",
        metadata_path.display()
    );
    Ok(())
}

/// 現在のディレクトリから`ipak`プロジェクトのメタデータを読み込みます。
///
/// この関数は、現在のディレクトリがプロジェクトのルートであると仮定し、
/// その中の`ipak/project.yaml`を読み込みます。
///
/// # Returns
/// `Ok(PackageData)` パースされたプロジェクトメタデータ。
/// `Err(io::Error)` ファイルが見つからない、読み込み、またはパースに失敗した場合。
pub fn from_current() -> Result<PackageData, io::Error> {
    let current_dir = env::current_dir()?;
    let metadata_path = current_dir.join("ipak/project.yaml");

    log::debug!("Attempting to read from: {}", metadata_path.display());

    if !is_file_exists(metadata_path.to_str().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "Invalid path characters in current directory",
        )
    })?) {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!(
                "'ipak/project.yaml' not found in current directory: {}",
                current_dir.display()
            ),
        ));
    }

    let read_data =
        std::fs::read_to_string(&metadata_path).map_err(|e| {
            io::Error::new(
                e.kind(),
                format!(
                    "Failed to read {}: {}",
                    metadata_path.display(),
                    e
                ),
            )
        })?;

    serde_yaml::from_str::<PackageData>(&read_data).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Failed to parse {}: {}", metadata_path.display(), e),
        )
    })
}

/// 指定された`PackageData`を現在のディレクトリの`ipak`プロジェクトメタデータファイルに書き込みます。
///
/// この関数は、現在のディレクトリがプロジェクトのルートであると仮定し、
/// その中の`ipak/project.yaml`に`PackageData`を書き込みます。
/// 必要な親ディレクトリが存在しない場合は作成します。
///
/// # Arguments
/// * `package_data` - 書き込む`PackageData`構造体への参照。
///
/// # Returns
/// `Ok(())` 成功した場合。
/// `Err(io::Error)` ファイルの書き込みまたはシリアライズに失敗した場合。
pub fn to_current(package_data: &PackageData) -> Result<(), io::Error> {
    let current_dir = env::current_dir()?;
    let metadata_path = current_dir.join("ipak/project.yaml");

    let parent_dir = metadata_path.parent().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "Could not determine parent directory for ipak/project.yaml",
        )
    })?;
    std::fs::create_dir_all(parent_dir)?;

    log::debug!("Attempting to write to: {}", metadata_path.display());

    let yaml_string =
        serde_yaml::to_string(package_data).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Failed to serialize PackageData to YAML: {}", e),
            )
        })?;

    std::fs::write(&metadata_path, yaml_string).map_err(|e| {
        io::Error::new(
            e.kind(),
            format!(
                "Failed to write to {}: {}",
                metadata_path.display(),
                e
            ),
        )
    })?;

    log::debug!(
        "Successfully wrote project metadata to {}",
        metadata_path.display()
    );
    Ok(())
}
