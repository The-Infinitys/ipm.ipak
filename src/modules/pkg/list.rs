//! このモジュールは、インストールされているパッケージのリストを管理します。
//! ローカルおよびグローバルなパッケージリストの読み込み、書き込み、追加、削除、表示機能を提供します。

use super::super::system::path;
use super::PackageData;
use crate::modules::project::ExecMode;
use crate::utils::color::colorize::*;
use crate::utils::error::Error;
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use serde_yaml;
use std::fmt::{self, Display, Formatter};
use std::fs;
use std::io;
use std::path::PathBuf;

/// パッケージリストのデータを表す構造体です。
#[derive(Serialize, Deserialize)]
pub struct PackageListData {
    /// 最終更新日時。
    pub last_modified: DateTime<Local>,
    /// インストールされているパッケージのリスト。
    pub installed_packages: Vec<InstalledPackageData>,
}

impl Default for PackageListData {
    /// デフォルトの`PackageListData`インスタンスを返します。
    /// 最終更新日時は現在時刻、インストール済みパッケージリストは空になります。
    fn default() -> Self {
        Self {
            last_modified: Local::now(),
            installed_packages: Vec::new(),
        }
    }
}

/// インストールされている個々のパッケージのデータを表す構造体です。
#[derive(Serialize, Deserialize, Default, Clone)]
pub struct InstalledPackageData {
    /// パッケージの基本情報。
    pub info: PackageData,
    /// 最終更新日時。
    pub last_modified: DateTime<Local>,
}

impl PackageListData {
    /// 指定されたファイルパスから`PackageListData`を読み込みます。
    ///
    /// ファイルが存在しない場合は、デフォルトの空のリストを返します。
    ///
    /// # Arguments
    /// * `list_filepath` - パッケージリストファイルへのパス。
    ///
    /// # Returns
    /// `Ok(PackageListData)` 読み込まれたパッケージリストデータ。
    /// `Err(io::Error)` ファイルの読み込みまたはパースに失敗した場合。
    fn from_filepath(
        list_filepath: &PathBuf,
    ) -> Result<PackageListData, io::Error> {
        let packageslist_str = match fs::read_to_string(list_filepath) {
            Ok(s) => s,
            Err(e) => {
                if e.kind() == io::ErrorKind::NotFound {
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
    /// `PackageListData`を整形して表示します。
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
    /// `InstalledPackageData`を整形して表示します。
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

/// 指定されたモードに基づいてインストール済みパッケージを一覧表示します。
///
/// # Arguments
/// * `mode` - 実行モード（ローカルまたはグローバル）。
///
/// # Returns
/// `Ok(())` パッケージリストが正常に表示された場合。
/// `Err(Error)` パッケージリストの取得または表示中にエラーが発生した場合。
pub fn list(mode: ExecMode) -> Result<(), Error> {
    let packages_list_data = match mode {
        ExecMode::Local => {
            get_local().map_err(|e| Error::from(e))?
        }
        ExecMode::Global => {
            get_global().map_err(|e| Error::from(e))?
        }
    };
    println!("{}", packages_list_data);
    Ok(())
}

/// ローカルのインストール済みパッケージリストを取得します。
///
/// # Returns
/// `Ok(PackageListData)` ローカルパッケージリストデータ。
/// `Err(std::io::Error)` ファイルの読み込みまたはパースに失敗した場合。
pub fn get_local() -> Result<PackageListData, std::io::Error> {
    let local_filepath = path::local::packageslist_filepath();
    PackageListData::from_filepath(&local_filepath)
}

/// グローバルのインストール済みパッケージリストを取得します。
///
/// # Returns
/// `Ok(PackageListData)` グローバルパッケージリストデータ。
/// `Err(std::io::Error)` ファイルの読み込みまたはパースに失敗した場合。
pub fn get_global() -> Result<PackageListData, std::io::Error> {
    let global_filepath = path::global::packageslist_filepath();
    PackageListData::from_filepath(&global_filepath)
}

/// ローカルのパッケージリストにデータを適用し、ファイルに保存します。
///
/// # Arguments
/// * `data` - 適用する`PackageListData`。
///
/// # Returns
/// `Ok(())` データが正常に適用され、保存された場合。
/// `Err(std::io::Error)` ファイルの書き込みまたはシリアライズに失敗した場合。
pub fn apply_local(
    mut data: PackageListData,
) -> Result<(), std::io::Error> {
    let local_filepath = path::local::packageslist_filepath();

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

    data.last_modified = Local::now();

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

/// グローバルのパッケージリストにデータを適用し、ファイルに保存します。
///
/// # Arguments
/// * `data` - 適用する`PackageListData`。
///
/// # Returns
/// `Ok(())` データが正常に適用され、保存された場合。
/// `Err(std::io::Error)` ファイルの書き込みまたはシリアライズに失敗した場合。
pub fn apply_global(
    mut data: PackageListData,
) -> Result<(), std::io::Error> {
    let global_filepath = path::global::packageslist_filepath();

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

    data.last_modified = Local::now();

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

/// ローカルのパッケージリストに新しいパッケージを追加します。
///
/// 同じ名前のパッケージが既に存在する場合は、そのデータを更新します。
///
/// # Arguments
/// * `new_pkg` - 追加する`InstalledPackageData`。
///
/// # Returns
/// `Ok(())` パッケージが正常に追加または更新された場合。
/// `Err(io::Error)` パッケージリストの読み込み、書き込み、または更新中にエラーが発生した場合。
pub fn add_pkg_local(
    new_pkg: InstalledPackageData,
) -> Result<(), io::Error> {
    let mut data = get_local()?;
    let mut found = false;
    for i in 0..data.installed_packages.len() {
        if data.installed_packages[i].info.about.package.name
            == new_pkg.info.about.package.name
        {
            data.installed_packages[i] = new_pkg.clone();
            found = true;
            eprintln!(
                "{} Package '{}' already exists locally. Updating its data.",
                "Info:".blue().bold(),
                data.installed_packages[i].info.about.package.name
            );
            break;
        }
    }

    if !found {
        data.installed_packages.push(new_pkg);
        eprintln!(
            "{} Package added to local list.",
            "Info:".blue().bold()
        );
    }

    apply_local(data)?;
    Ok(())
}

/// グローバルのパッケージリストに新しいパッケージを追加します。
///
/// 同じ名前のパッケージが既に存在する場合は、そのデータを更新します。
///
/// # Arguments
/// * `new_pkg` - 追加する`InstalledPackageData`。
///
/// # Returns
/// `Ok(())` パッケージが正常に追加または更新された場合。
/// `Err(io::Error)` パッケージリストの読み込み、書き込み、または更新中にエラーが発生した場合。
pub fn add_pkg_global(
    new_pkg: InstalledPackageData,
) -> Result<(), io::Error> {
    let mut data = get_global()?;
    let mut found = false;
    for i in 0..data.installed_packages.len() {
        if data.installed_packages[i].info.about.package.name
            == new_pkg.info.about.package.name
        {
            data.installed_packages[i] = new_pkg.clone();
            found = true;
            eprintln!(
                "{} Package '{}' already exists globally. Updating its data.",
                "Info:".blue().bold(),
                data.installed_packages[i].info.about.package.name
            );
            break;
        }
    }

    if !found {
        data.installed_packages.push(new_pkg);
        eprintln!(
            "{} Package added to global list.",
            "Info:".blue().bold()
        );
    }

    apply_global(data)?;
    Ok(())
}

/// ローカルのパッケージリストから指定されたパッケージを削除します。
///
/// # Arguments
/// * `package_name` - 削除するパッケージの名前。
///
/// # Returns
/// `Ok(true)` パッケージが正常に削除された場合。
/// `Ok(false)` パッケージが見つからなかった場合。
/// `Err(io::Error)` パッケージリストの読み込み、書き込み、または更新中にエラーが発生した場合。
pub fn del_pkg_local(package_name: &str) -> Result<bool, io::Error> {
    let mut data = get_local()?;
    let initial_len = data.installed_packages.len();
    data.installed_packages
        .retain(|pkg| pkg.info.about.package.name != package_name);

    if data.installed_packages.len() < initial_len {
        apply_local(data)?;
        Ok(true)
    } else {
        eprintln!(
            "{} Package '{}' not found in local installations.",
            "Warning:".yellow().bold(),
            package_name
        );
        Ok(false)
    }
}

/// グローバルのパッケージリストから指定されたパッケージを削除します。
///
/// # Arguments
/// * `package_name` - 削除するパッケージの名前。
///
/// # Returns
/// `Ok(true)` パッケージが正常に削除された場合。
/// `Ok(false)` パッケージが見つからなかった場合。
/// `Err(io::Error)` パッケージリストの読み込み、書き込み、または更新中にエラーが発生した場合。
pub fn del_pkg_global(package_name: &str) -> Result<bool, io::Error> {
    let mut data = get_global()?;
    let initial_len = data.installed_packages.len();
    data.installed_packages
        .retain(|pkg| pkg.info.about.package.name != package_name);

    if data.installed_packages.len() < initial_len {
        apply_global(data)?;
        Ok(true)
    } else {
        eprintln!(
            "{} Package '{}' not found in global installations.",
            "Warning:".yellow().bold(),
            package_name
        );
        Ok(false)
    }
}
