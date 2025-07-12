//! このモジュールは、既存のプロジェクトを`ipak`プロジェクトとして初期化する機能を提供します。
//! プロジェクトの言語を検出し、それに応じた`ipak`スクリプトと設定ファイルを生成します。

use super::metadata;
use crate::utils::files::file_creation;
use crate::utils::version::Version;
use std::env;
use std::fmt;
use std::fs;
use std::path::Path;
use std::str::FromStr;

/// 検出されたパッケージの言語を表す列挙型です。
enum PackageLanguage {
    Python,
    Rust,
    DotNet,
    Other,
}

impl fmt::Display for PackageLanguage {
    /// `PackageLanguage`を整形して表示します。
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PackageLanguage::Python => write!(f, "python"),
            PackageLanguage::Rust => write!(f, "rust"),
            PackageLanguage::DotNet => write!(f, "dotnet"),
            PackageLanguage::Other => write!(f, "other"),
        }
    }
}

/// セットアップするファイルとその内容を定義する構造体です。
struct SetUpItem {
    /// ファイルのパス。
    path: String,
    /// ファイルに書き込む内容。
    content: String,
}

/// テンプレートファイルを生成します。
///
/// 指定された`SetUpItem`のリストに基づいて、ファイルを作成し内容を書き込みます。
///
/// # Arguments
/// * `setup_list` - 作成するファイルと内容のリスト。
///
/// # Returns
/// `Ok(())` 成功した場合。
/// `Err(std::io::Error)` ファイル作成中にエラーが発生した場合。
fn setup_template_files(
    setup_list: Vec<SetUpItem>,
) -> Result<(), std::io::Error> {
    for item in setup_list {
        file_creation(&item.path, &item.content).map_err(|e| {
            std::io::Error::new(
                e.kind(),
                format!(
                    "Failed to create template file '{}': {}",
                    item.path, e
                ),
            )
        })?;
    }
    log::debug!("Successfully set up template files.");
    Ok(())
}

/// 既存のプロジェクトを`ipak`プロジェクトとして初期化します。
///
/// 現在のディレクトリをスキャンし、`Cargo.toml`, `pyproject.toml`, `.csproj`ファイルなどから
/// プロジェクトの言語を検出します。検出された言語に基づいて、`ipak/project.yaml`を更新し、
/// 適切な`ipak`スクリプト（ビルド、インストール、削除、パージ）と設定ファイルを生成します。
///
/// # Returns
/// `Ok(())` 初期化が正常に完了した場合。
/// `Err(std::io::Error)` ファイル操作、メタデータ処理、または言語検出中にエラーが発生した場合。
pub fn init() -> Result<(), std::io::Error> {
    let mut pkg_metadata = metadata::from_current().unwrap_or_default();
    let target_dir = env::current_dir()?;
    let readme_path = target_dir.join("README.md");

    if readme_path.exists() {
        let readme_content = fs::read_to_string(readme_path)?;
        pkg_metadata.about.package.description = readme_content;
        log::debug!("Initialized project metadata.");
    }

    let mut pkg_lang = PackageLanguage::Other;
    let mut lang_file_path_str = String::new();

    if target_dir.join("Cargo.toml").exists() {
        pkg_lang = PackageLanguage::Rust;
        lang_file_path_str =
            target_dir.join("Cargo.toml").to_string_lossy().into_owned();
    } else if target_dir.join("pyproject.toml").exists() {
        pkg_lang = PackageLanguage::Python;
        lang_file_path_str = target_dir
            .join("pyproject.toml")
            .to_string_lossy()
            .into_owned();
    } else {
        let dotnet_result = find_csproj_file_recursive(&target_dir)?;
        if let Some(csproj_path) = dotnet_result {
            pkg_lang = PackageLanguage::DotNet;
            lang_file_path_str =
                csproj_path.to_string_lossy().into_owned();
        }
    }

    log::debug!("Detected package language: {}", pkg_lang);

    match pkg_lang {
        PackageLanguage::Rust => {
            if !lang_file_path_str.is_empty() {
                if let Some((name, version)) =
                    parse_cargo_toml(Path::new(&lang_file_path_str))?
                {
                    pkg_metadata.about.package.name = name;
                    pkg_metadata.about.package.version =
                        Version::from_str(&version).unwrap_or_default();
                }
            }
        }
        PackageLanguage::Python => {
            if !lang_file_path_str.is_empty() {
                if let Some((name, version)) =
                    parse_pyproject_toml(Path::new(&lang_file_path_str))?
                {
                    pkg_metadata.about.package.name = name;
                    pkg_metadata.about.package.version =
                        Version::from_str(&version).unwrap_or_default();
                }
            }
        }
        PackageLanguage::DotNet => {
            if !lang_file_path_str.is_empty() {
                if let Some((name, version)) =
                    parse_csproj(Path::new(&lang_file_path_str))?
                {
                    pkg_metadata.about.package.name = name;
                    pkg_metadata.about.package.version =
                        Version::from_str(&version).unwrap_or_default();
                }
            }
        }
        PackageLanguage::Other => {
            log::debug!(
                "No specific package language detected, skipping name and version extraction."
            );
        }
    }
    metadata::to_current(&pkg_metadata)?;
    log::debug!(
        "Project metadata initialized/updated in ipak/project.yaml."
    );

    log::debug!("Setting up ipak scripts based on detected language...");

    let script_readme_content =
        include_str!("create/templates/script-README.md").to_string();

    let script_setup_result = match pkg_lang {
        PackageLanguage::Rust => {
            let setup_list = vec![
                SetUpItem {
                    path: "ipak/scripts/build.sh".to_string(),
                    content: include_str!(
                        "create/templates/rust/ipak/scripts/build.sh"
                    )
                    .to_string(),
                },
                SetUpItem {
                    path: "ipak/scripts/install.sh".to_string(),
                    content: include_str!(
                        "create/templates/rust/ipak/scripts/install.sh"
                    )
                    .to_string(),
                },
                SetUpItem {
                    path: "ipak/scripts/remove.sh".to_string(),
                    content: include_str!(
                        "create/templates/rust/ipak/scripts/remove.sh"
                    )
                    .to_string(),
                },
                SetUpItem {
                    path: "ipak/scripts/purge.sh".to_string(),
                    content: include_str!(
                        "create/templates/rust/ipak/scripts/purge.sh"
                    )
                    .to_string(),
                },
                SetUpItem {
                    path: "ipak/project-ignore.yaml".to_string(),
                    content: include_str!(
                        "create/templates/rust/ipak/project-ignore.yaml"
                    )
                    .to_string(),
                },
                SetUpItem {
                    path: "ipak/scripts/README.md".to_string(),
                    content: script_readme_content.clone(),
                },
            ];
            setup_template_files(setup_list)
        }
        PackageLanguage::Python => {
            let setup_list = vec![
                SetUpItem {
                    path: "ipak/scripts/build.sh".to_string(),
                    content: include_str!(
                        "create/templates/python/ipak/scripts/build.sh"
                    )
                    .to_string(),
                },
                SetUpItem {
                    path: "ipak/scripts/install.sh".to_string(),
                    content: include_str!(
                        "create/templates/python/ipak/scripts/install.sh"
                    )
                    .to_string(),
                },
                SetUpItem {
                    path: "ipak/scripts/remove.sh".to_string(),
                    content: include_str!(
                        "create/templates/python/ipak/scripts/remove.sh"
                    )
                    .to_string(),
                },
                SetUpItem {
                    path: "ipak/scripts/purge.sh".to_string(),
                    content: include_str!(
                        "create/templates/python/ipak/scripts/purge.sh"
                    )
                    .to_string(),
                },
                SetUpItem {
                    path: "ipak/project-ignore.yaml".to_string(),
                    content: include_str!(
                        "create/templates/python/ipak/project-ignore.yaml"
                    )
                    .to_string(),
                },
                SetUpItem {
                    path: "ipak/scripts/README.md".to_string(),
                    content: script_readme_content.clone(),
                },
            ];
            setup_template_files(setup_list)
        }
        PackageLanguage::DotNet => {
            let setup_list = vec![
                SetUpItem {
                    path: "ipak/scripts/build.sh".to_string(),
                    content: include_str!(
                        "create/templates/dotnet/ipak/scripts/build.sh"
                    )
                    .to_string(),
                },
                SetUpItem {
                    path: "ipak/scripts/install.sh".to_string(),
                    content: include_str!(
                        "create/templates/dotnet/ipak/scripts/install.sh"
                    )
                    .to_string(),
                },
                SetUpItem {
                    path: "ipak/scripts/remove.sh".to_string(),
                    content: include_str!(
                        "create/templates/dotnet/ipak/scripts/remove.sh"
                    )
                    .to_string(),
                },
                SetUpItem {
                    path: "ipak/scripts/purge.sh".to_string(),
                    content: include_str!(
                        "create/templates/dotnet/ipak/scripts/purge.sh"
                    )
                    .to_string(),
                },
                SetUpItem {
                    path: "ipak/project-ignore.yaml".to_string(),
                    content: include_str!(
                        "create/templates/dotnet/ipak/project-ignore.yaml"
                    )
                    .to_string(),
                },
                SetUpItem {
                    path: "ipak/scripts/README.md".to_string(),
                    content: script_readme_content.clone(),
                },
            ];
            setup_template_files(setup_list)
        }
        PackageLanguage::Other => {
            let setup_list = vec![
                SetUpItem {
                    path: "ipak/scripts/build.sh".to_string(),
                    content: include_str!(
                        "create/templates/default/ipak/scripts/build.sh"
                    )
                    .to_string(),
                },
                SetUpItem {
                    path: "ipak/scripts/install.sh".to_string(),
                    content: include_str!(
                        "create/templates/default/ipak/scripts/install.sh"
                    )
                    .to_string(),
                },
                SetUpItem {
                    path: "ipak/scripts/remove.sh".to_string(),
                    content: include_str!(
                        "create/templates/default/ipak/scripts/remove.sh"
                    )
                    .to_string(),
                },
                SetUpItem {
                    path: "ipak/scripts/purge.sh".to_string(),
                    content: include_str!(
                        "create/templates/default/ipak/scripts/purge.sh"
                    )
                    .to_string(),
                },
                SetUpItem {
                    path: "ipak/scripts/README.md".to_string(),
                    content: script_readme_content,
                },
            ];
            setup_template_files(setup_list)
        }
    };

    script_setup_result?;

    log::debug!("ipak init process completed successfully.");
    Ok(())
}

/// 指定されたディレクトリ内で`.csproj`ファイルを再帰的に検索します。
///
/// 特定のディレクトリ（`target`, `node_modules`, `bin`, `obj`）は検索から除外されます。
///
/// # Arguments
/// * `dir` - 検索を開始するディレクトリ。
///
/// # Returns
/// `Ok(Some(PathBuf))` `.csproj`ファイルが見つかった場合、そのパス。
/// `Ok(None)` `.csproj`ファイルが見つからなかった場合。
/// `Err(std::io::Error)` ディレクトリの読み取り中にエラーが発生した場合。
fn find_csproj_file_recursive(
    dir: &Path,
) -> Result<Option<std::path::PathBuf>, std::io::Error> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            if path.extension().is_some_and(|ext| ext == "csproj") {
                return Ok(Some(path));
            }
        } else if path.is_dir() {
            if path.file_name().is_some_and(|name| {
                name == "target"
                    || name == "node_modules"
                    || name == "bin"
                    || name == "obj"
            }) {
                continue;
            }
            if let Some(csproj_path) = find_csproj_file_recursive(&path)? {
                return Ok(Some(csproj_path));
            }
        }
    }
    Ok(None)
}

/// `Cargo.toml`ファイルからパッケージ名とバージョンをパースします。
///
/// # Arguments
/// * `path` - `Cargo.toml`ファイルへのパス。
///
/// # Returns
/// `Ok(Some((name, version)))` パッケージ名とバージョンが見つかった場合。
/// `Ok(None)` パッケージ名またはバージョンが見つからなかった場合。
/// `Err(std::io::Error)` ファイルの読み取りまたはTOMLのパースに失敗した場合。
fn parse_cargo_toml(
    path: &Path,
) -> Result<Option<(String, String)>, std::io::Error> {
    let content = fs::read_to_string(path)?;
    if let Ok(toml_doc) = content.parse::<toml::Value>() {
        if let Some(package) = toml_doc.get("package") {
            let name = package
                .get("name")
                .and_then(|n| n.as_str())
                .map(|s| s.to_string());
            let version = package
                .get("version")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            if let (Some(name), Some(version)) = (name, version) {
                return Ok(Some((name, version)));
            }
        }
    }
    Ok(None)
}

/// `pyproject.toml`ファイルからプロジェクト名とバージョンをパースします。
///
/// # Arguments
/// * `path` - `pyproject.toml`ファイルへのパス。
///
/// # Returns
/// `Ok(Some((name, version)))` プロジェクト名とバージョンが見つかった場合。
/// `Ok(None)` プロジェクト名またはバージョンが見つからなかった場合。
/// `Err(std::io::Error)` ファイルの読み取りまたはTOMLのパースに失敗した場合。
fn parse_pyproject_toml(
    path: &Path,
) -> Result<Option<(String, String)>, std::io::Error> {
    let content = fs::read_to_string(path)?;
    if let Ok(toml_doc) = content.parse::<toml::Value>() {
        if let Some(project) = toml_doc.get("project") {
            let name = project
                .get("name")
                .and_then(|n| n.as_str())
                .map(|s| s.to_string());
            let version = project
                .get("version")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            if let (Some(name), Some(version)) = (name, version) {
                return Ok(Some((name, version)));
            }
        }
    }
    Ok(None)
}

/// `.csproj`ファイルからアセンブリ名とバージョンをパースします。
///
/// XMLを直接パースするのではなく、タグの文字列検索によって情報を抽出します。
///
/// # Arguments
/// * `path` - `.csproj`ファイルへのパス。
///
/// # Returns
/// `Ok(Some((name, version)))` アセンブリ名とバージョンが見つかった場合。
/// `Ok(None)` アセンブリ名またはバージョンが見つからなかった場合。
/// `Err(std::io::Error)` ファイルの読み取りに失敗した場合。
fn parse_csproj(
    path: &Path,
) -> Result<Option<(String, String)>, std::io::Error> {
    let content = fs::read_to_string(path)?;

    let name_tag_start = "<AssemblyName>";
    let name_tag_end = "</AssemblyName>";
    let version_tag_start = "<Version>";
    let version_tag_end = "</Version>";

    let mut name: Option<String> = None;
    let mut version: Option<String> = None;

    if let Some(start) = content.find(name_tag_start) {
        if let Some(end) = content[start..].find(name_tag_end) {
            name = Some(
                content[start + name_tag_start.len()..start + end]
                    .trim()
                    .to_string(),
            );
        }
    }

    if let Some(start) = content.find(version_tag_start) {
        if let Some(end) = content[start..].find(version_tag_end) {
            version = Some(
                content[start + version_tag_start.len()..start + end]
                    .trim()
                    .to_string(),
            );
        }
    }

    if let (Some(name_val), Some(version_val)) = (name, version) {
        Ok(Some((name_val, version_val)))
    } else {
        Ok(None)
    }
}
