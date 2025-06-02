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
    // from_current を呼び出して PackageData を取得し、それを表示する
    let package_data = from_current()?;
    println!("{}", package_data);
    Ok(())
}

/// `PackageData` 構造体を `project.yaml` ファイルにシリアライズして書き込みます。
/// ファイルが存在しない場合は新しく作成し、存在する場合は上書きします。
///
/// # 引数
/// * `package_data` - 書き込む `PackageData` 構造体への参照。
///
/// # 戻り値
/// 書き込みに成功した場合は `Ok(())` を返します。
/// ファイルのパス取得、ディレクトリ作成、シリアライズ、またはファイル書き込みに失敗した場合は
/// `io::Error` を `Err` で返します。
pub fn write(package_data: &PackageData) -> Result<(), io::Error> {
    let metadata_path = get_path()?; // project.yaml へのパスを取得

    // 親ディレクトリが存在しない場合は作成します。
    // `std::fs::write` はファイルが存在しない場合に作成しますが、その前にディレクトリが必要です。
    let parent_dir = metadata_path.parent().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "Could not determine parent directory for project.yaml",
        )
    })?;
    std::fs::create_dir_all(parent_dir)?; // ディレクトリが存在しない場合は作成

    // PackageData を YAML 文字列にシリアライズ
    let yaml_string = serde_yaml::to_string(package_data).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Failed to serialize PackageData to YAML: {}", e),
        )
    })?;

    // YAML 文字列を project.yaml ファイルに書き込み
    // std::fs::write はファイルが存在しない場合に作成します。
    std::fs::write(&metadata_path, yaml_string).map_err(|e| {
        io::Error::new(
            e.kind(),
            format!("Failed to write to {}: {}", metadata_path.display(), e),
        )
    })?;

    dprintln!("Successfully wrote project metadata to {}", metadata_path.display());
    Ok(())
}

/// カレントディレクトリにある `ipak/project.yaml` ファイルのメタデータを読み込みます。
///
/// # 戻り値
/// パースされた `PackageData` を `Ok` で返します。
/// ファイルが見つからない、読み込みに失敗する、またはパースに失敗する場合は `io::Error` を `Err` で返します。
pub fn from_current() -> Result<PackageData, io::Error> {
    let current_dir = env::current_dir()?;
    let metadata_path = current_dir.join("ipak/project.yaml");

    dprintln!("Attempting to read from: {}", metadata_path.display());

    // ファイルの存在チェック（オプション：エラーメッセージをより具体的にするため）
    if !is_file_exists(metadata_path.to_str().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "Invalid path characters in current directory",
        )
    })?) {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("'ipak/project.yaml' not found in current directory: {}", current_dir.display()),
        ));
    }

    // ファイルを読み込む
    let read_data = std::fs::read_to_string(&metadata_path).map_err(|e| {
        io::Error::new(
            e.kind(),
            format!(
                "Failed to read {}: {}",
                metadata_path.display(),
                e
            ),
        )
    })?;

    // YAML を PackageData にパースする
    serde_yaml::from_str::<PackageData>(&read_data).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Failed to parse {}: {}", metadata_path.display(), e),
        )
    })
}