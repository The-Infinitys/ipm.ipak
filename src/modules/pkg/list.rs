use super::super::system::path;
use super::PackageData;
use crate::utils::color::colorize::*;
use crate::utils::shell;
use chrono::{DateTime, Local};
use cmd_arg::cmd_arg::Option;
use serde::{Deserialize, Serialize};
use serde_yaml;
use std::fmt::{self, Display, Formatter};
use std::fs;
use std::io;
use std::path::PathBuf;

/// インストールされているパッケージリストのメタデータを表します。
#[derive(Serialize, Deserialize)]
pub struct PackageListData {
    pub last_modified: DateTime<Local>,
    pub installed_packages: Vec<InstalledPackageData>,
}

impl Default for PackageListData {
    fn default() -> Self {
        Self {
            last_modified: Local::now(),
            installed_packages: Vec::new(),
        }
    }
}

/// 個々のインストール済みパッケージの詳細情報を表します。
#[derive(Serialize, Deserialize, Default, Clone)]
pub struct InstalledPackageData {
    pub info: PackageData,
    pub last_modified: DateTime<Local>,
}

impl PackageListData {
    /// 指定されたパスからパッケージリストデータを読み込みます。
    ///
    /// ファイルが存在しない場合は、空の `PackageListData` を返します。
    ///
    /// # 引数
    /// * `list_filepath` - 読み込むパッケージリストファイルのパス。
    ///
    /// # 戻り値
    /// 読み込みとパースが成功した場合は `PackageListData` を、
    /// それ以外のI/Oエラーやパースエラーが発生した場合は `io::Error` を返します。
    fn from_filepath(
        list_filepath: &PathBuf,
    ) -> Result<PackageListData, io::Error> {
        let packageslist_str = match fs::read_to_string(list_filepath) {
            Ok(s) => s,
            Err(e) => {
                if e.kind() == io::ErrorKind::NotFound {
                    // ファイルが存在しない場合は空のリストを返す
                    return Ok(PackageListData::default());
                } else {
                    return Err(io::Error::new(
                        e.kind(),
                        format!(
                            "Failed to read packages list file '{}': {}",
                            list_filepath.display(),
                            e
                        ),
                    ));
                }
            }
        };

        serde_yaml::from_str(&packageslist_str).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "Failed to parse packages list file '{}': {}",
                    list_filepath.display(),
                    e
                ),
            )
        })
    }
}

impl Display for PackageListData {
    /// `PackageListData` を整形して表示します。
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        writeln!(
            f,
            "{}: {}",
            "Last Modified".green().bold(),
            self.last_modified.to_rfc3339()
        )?;
        writeln!(f, "{}:", "Packages".cyan().bold())?;
        if self.installed_packages.is_empty() {
            writeln!(f, "  No packages installed in this scope.")?;
        } else {
            for pkg in &self.installed_packages {
                writeln!(f, "{}", pkg)?;
            }
        }
        Ok(())
    }
}

impl Display for InstalledPackageData {
    /// `InstalledPackageData` を整形して表示します。
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        writeln!(
            f,
            "  {}: {}",
            "Name".bold(),
            self.info.about.package.name.cyan()
        )?;
        writeln!(
            f,
            "    {}: {}",
            "Version".bold(),
            self.info.about.package.version
        )?;
        writeln!(
            f,
            "    {}: {} <{}>",
            "Author".bold(),
            self.info.about.author.name,
            self.info.about.author.email
        )?;
        writeln!(
            f,
            "    {}: {}",
            "Last Modified".bold(),
            self.last_modified.to_rfc3339()
        )?;
        if !self.info.relation.is_empty() {
            writeln!(f, "    {}", "Relations:".bold())?;
            let mut indented_relations = String::new();
            for line in format!("{}", self.info.relation).lines() {
                indented_relations.push_str(&format!("      {}\n", line));
            }
            write!(f, "{}", indented_relations)?;
        }
        Ok(())
    }
}

/// インストールされているパッケージのリストを表示します。
///
/// この関数は、`--local` または `--global` オプションに基づいて、
/// ローカルまたはグローバルなパッケージリストを読み込み、表示します。
/// デフォルトでは、現在のユーザーがスーパーユーザーでない限りローカルリストを表示します。
///
/// # 引数
/// * `args` - コマンドライン引数のリスト。
///
/// # 戻り値
/// リスト表示が成功した場合は `Ok(())` を、エラーが発生した場合は `std::io::Error` を返します。
pub fn list(args: Vec<&Option>) -> Result<(), std::io::Error> {
    // デフォルトのリストターゲットは、現在のユーザーがスーパーユーザーでない場合ローカル
    let mut list_local = !shell::is_superuser();

    // 引数を解析してリストターゲットを決定
    for arg in args {
        match arg.opt_str.as_str() {
            "--local" | "-l" => list_local = true,
            "--global" | "-g" => list_local = false,
            _ => {
                eprintln!(
                    "{} Unknown option '{}'. Ignoring.",
                    "Warning:".yellow().bold(),
                    arg.opt_str
                );
            }
        }
    }

    let packages_list_data =
        if list_local { get_local()? } else { get_global()? };

    println!("{}", packages_list_data);
    Ok(())
}

/// ローカルスコープのパッケージリストデータを取得します。
///
/// `~/.local/share/<your_app_name>/packages.yml` からパッケージリストを読み込みます。
/// ファイルが存在しない場合は、空の `PackageListData` を返します。
///
/// # 戻り値
/// 読み込みとパースが成功した場合は `PackageListData` を、失敗した場合は `io::Error` を返します。
pub fn get_local() -> Result<PackageListData, std::io::Error> {
    let local_filepath = path::local::packageslist_filepath();
    PackageListData::from_filepath(&local_filepath)
}

/// グローバルスコープのパッケージリストデータを取得します。
///
/// `/usr/local/share/<your_app_name>/packages.yml` からパッケージリストを読み込みます。
/// ファイルが存在しない場合は、空の `PackageListData` を返します。
///
/// # 戻り値
/// 読み込みとパースが成功した場合は `PackageListData` を、失敗した場合は `io::Error` を返します。
pub fn get_global() -> Result<PackageListData, std::io::Error> {
    let global_filepath = path::global::packageslist_filepath();
    PackageListData::from_filepath(&global_filepath)
}

// --- ここから新しい関数 ---

/// ローカルスコープのパッケージリストデータを保存します。
///
/// 指定された `PackageListData` をYAML形式でシリアライズし、
/// `~/.local/share/<your_app_name>/packages.yml` に書き込みます。
/// 書き込む直前に `last_modified` タイムスタンプが更新されます。
///
/// # 引数
/// * `data` - 保存するパッケージリストデータ。
///
/// # 戻り値
/// 保存が成功した場合は `Ok(())` を、失敗した場合は `io::Error` を返します。
pub fn apply_local(
    mut data: PackageListData,
) -> Result<(), std::io::Error> {
    let local_filepath = path::local::packageslist_filepath();

    // 親ディレクトリが存在しない場合に備えて作成
    if let Some(parent_dir) = local_filepath.parent() {
        fs::create_dir_all(parent_dir).map_err(|e| {
            io::Error::new(
                e.kind(),
                format!(
                    "Failed to create parent directory for '{}': {}",
                    local_filepath.display(),
                    e
                ),
            )
        })?;
    }

    // 最終更新日時を現在に設定
    data.last_modified = Local::now();

    // PackageListData をYAML文字列にシリアライズ
    let yaml_string = serde_yaml::to_string(&data).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "Failed to serialize package list data for '{}': {}",
                local_filepath.display(),
                e
            ),
        )
    })?;

    // ファイルに書き込み
    fs::write(&local_filepath, yaml_string).map_err(|e| {
        io::Error::new(
            e.kind(),
            format!(
                "Failed to write package list data to '{}': {}",
                local_filepath.display(),
                e
            ),
        )
    })?;

    Ok(())
}

/// グローバルスコープのパッケージリストデータを保存します。
///
/// 指定された `PackageListData` をYAML形式でシリアライズし、
/// `/usr/local/share/<your_app_name>/packages.yml` に書き込みます。
/// 書き込む直前に `last_modified` タイムスタンプが更新されます。
///
/// # 引数
/// * `data` - 保存するパッケージリストデータ。
///
/// # 戻り値
/// 保存が成功した場合は `Ok(())` を、失敗した場合は `io::Error` を返します。
pub fn apply_global(
    mut data: PackageListData,
) -> Result<(), std::io::Error> {
    let global_filepath = path::global::packageslist_filepath();

    // 親ディレクトリが存在しない場合に備えて作成
    if let Some(parent_dir) = global_filepath.parent() {
        fs::create_dir_all(parent_dir).map_err(|e| {
            io::Error::new(
                e.kind(),
                format!(
                    "Failed to create parent directory for '{}': {}",
                    global_filepath.display(),
                    e
                ),
            )
        })?;
    }

    // 最終更新日時を現在に設定
    data.last_modified = Local::now();

    // PackageListData をYAML文字列にシリアライズ
    let yaml_string = serde_yaml::to_string(&data).map_err(|e| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "Failed to serialize package list data for '{}': {}",
                global_filepath.display(),
                e
            ),
        )
    })?;

    // ファイルに書き込み
    fs::write(&global_filepath, yaml_string).map_err(|e| {
        io::Error::new(
            e.kind(),
            format!(
                "Failed to write package list data to '{}': {}",
                global_filepath.display(),
                e
            ),
        )
    })?;

    Ok(())
}

pub fn add_pkg_local(
    new_pkg: InstalledPackageData,
) -> Result<(), io::Error> {
    let mut data = get_local()?;
    let mut found = false;
    for i in 0..data.installed_packages.len() {
        if data.installed_packages[i].info.about.package.name
            == new_pkg.info.about.package.name
        {
            // 既存のデータを上書き
            data.installed_packages[i] = new_pkg.clone();
            found = true;
            eprintln!(
                "{} Package '{}' already exists locally. Updating its data.",
                "Info:".blue().bold(), // 情報メッセージに変更
                data.installed_packages[i].info.about.package.name
            );
            break;
        }
    }

    if !found {
        // 存在しない場合は新しく追加
        data.installed_packages.push(new_pkg);
        eprintln!(
            "{} Package added to local list.",
            "Info:".blue().bold()
        );
    }

    apply_local(data)?;
    Ok(())
}
pub fn add_pkg_global(
    new_pkg: InstalledPackageData,
) -> Result<(), io::Error> {
    let mut data = get_global()?;
    let mut found = false;
    for i in 0..data.installed_packages.len() {
        if data.installed_packages[i].info.about.package.name
            == new_pkg.info.about.package.name
        {
            // 既存のデータを上書き
            data.installed_packages[i] = new_pkg.clone();
            found = true;
            eprintln!(
                "{} Package '{}' already exists globally. Updating its data.",
                "Info:".blue().bold(), // 情報メッセージに変更
                data.installed_packages[i].info.about.package.name
            );
            break;
        }
    }

    if !found {
        // 存在しない場合は新しく追加
        data.installed_packages.push(new_pkg);
        eprintln!(
            "{} Package added to global list.",
            "Info:".blue().bold()
        );
    }

    apply_global(data)?;
    Ok(())
}
/// ローカルスコープのパッケージリストから指定された名前のパッケージを削除します。
///
/// パッケージがリストから削除された場合、ローカルのパッケージリストファイル
/// (`~/.local/share/<your_app_name>/packages.yml`) を更新します。
///
/// # 引数
/// * `package_name` - 削除するパッケージの名前。
///
/// # 戻り値
/// 削除が成功した場合は `Ok(true)` を、パッケージが見つからなかった場合は `Ok(false)` を、
/// エラーが発生した場合は `io::Error` を返します。
pub fn del_pkg_local(package_name: &str) -> Result<bool, io::Error> {
    let mut data = get_local()?;
    let initial_len = data.installed_packages.len();
    data.installed_packages
        .retain(|pkg| pkg.info.about.package.name != package_name);

    if data.installed_packages.len() < initial_len {
        // パッケージが削除された場合のみ保存
        apply_local(data)?;
        Ok(true)
    } else {
        eprintln!(
            "{} Package '{}' not found in local installations.",
            "Warning:".yellow().bold(),
            package_name
        );
        Ok(false) // パッケージが見つからなかった
    }
}
/// グローバルスコープのパッケージリストから指定された名前のパッケージを削除します。
///
/// パッケージがリストから削除された場合、グローバルのパッケージリストファイル
/// (`/usr/local/share/<your_app_name>/packages.yml`) を更新します。
///
/// # 引数
/// * `package_name` - 削除するパッケージの名前。
///
/// # 戻り値
/// 削除が成功した場合は `Ok(true)` を、パッケージが見つからなかった場合は `Ok(false)` を、
/// エラーが発生した場合は `io::Error` を返します。
pub fn del_pkg_global(package_name: &str) -> Result<bool, io::Error> {
    let mut data = get_global()?;
    let initial_len = data.installed_packages.len();
    data.installed_packages
        .retain(|pkg| pkg.info.about.package.name != package_name);

    if data.installed_packages.len() < initial_len {
        // パッケージが削除された場合のみ保存
        apply_global(data)?;
        Ok(true)
    } else {
        eprintln!(
            "{} Package '{}' not found in global installations.",
            "Warning:".yellow().bold(),
            package_name
        );
        Ok(false) // パッケージが見つからなかった
    }
}
