use crate::dprintln;
use crate::{modules::pkg::PackageData, utils::files::is_file_exists};
use std::{env, io, path::PathBuf}; // io::Error をインポート

/// 現在のディレクトリまたは親ディレクトリから `project.yaml` を含むプロジェクトのルートディレクトリを探します。
///
/// # 戻り値
/// `project.yaml` が見つかった場合はそのディレクトリの `PathBuf` を `Ok` で返します。
/// 見つからなかった場合は `io::Error` を `Err` で返します。
pub fn get_dir() -> Result<PathBuf, io::Error> {
    let mut current_path = env::current_dir()?; // Result を直接扱う
    loop {
        let metadata_path = current_path.join("ipak/project.yaml");
        dprintln!("{}", metadata_path.display()); // .to_str().unwrap() を避ける
        if is_file_exists(metadata_path.to_str().ok_or_else(|| {
            // .to_str() の失敗を考慮
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "Invalid path characters",
            )
        })?) {
            return Ok(current_path);
        } else {
            dprintln!(
                "Not found project.yaml in {}",
                current_path.display()
            );
            if let Some(parent) = current_path.parent() {
                current_path = parent.to_owned(); // 親ディレクトリに移動
            } else {
                // ルートディレクトリに到達し、project.yaml が見つからなかった場合
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    "project.yaml not found in current or parent directories",
                ));
            }
        }
    }
}

/// プロジェクトの `project.yaml` ファイルへのパスを取得します。
///
/// # 戻り値
/// `project.yaml` への `PathBuf` を `Ok` で返します。
/// ファイルが見つからない場合は `io::Error` を `Err` で返します。
pub fn get_path() -> Result<PathBuf, io::Error> {
    get_dir().map(|dir| dir.join("ipak/project.yaml"))
}

/// `project.yaml` ファイルを読み込み、`PackageData` 構造体にパースします。
///
/// # 戻り値
/// パースされた `PackageData` を `Ok` で返します。
/// ファイルの読み込みやパースに失敗した場合は `io::Error` を `Err` で返します。
pub fn metadata() -> Result<PackageData, io::Error> {
    let metadata_path = get_path()?; // get_path() のエラーを伝播
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

/// プロジェクトのメタデータを読み込み、標準出力に表示します。
///
/// # 戻り値
/// メタデータの表示に成功した場合は `Ok(())` を返します。
/// メタデータの取得や表示に失敗した場合は `io::Error` を `Err` で返します。
pub fn show_metadata() -> Result<(), io::Error> {
    let package_data = metadata()?; // metadata() のエラーを伝播
    println!("{}", package_data);
    Ok(())
}
