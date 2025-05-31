use std::io;
use std::str::FromStr;
use thiserror::Error;
mod templates;
use super::super::pkg::{AuthorAboutData, PackageData}; // 複数のアイテムを一行でインポート
use crate::utils::files::file_creation;
use colored::Colorize;
use std::fmt::{self, Display, Formatter};
/// プロジェクトテンプレートのタイプを定義します。
#[derive(PartialEq, Eq, Default)] // Default を追加して、ProjectParams のデフォルト実装を容易にする
pub enum ProjectTemplateType {
    #[default]
    // Default トレイトの実装でデフォルトを Default に設定
    Default,
    Rust,
    Python,
    Dotnet,
    CLang,
}

impl FromStr for ProjectTemplateType {
    type Err = String;

    /// 文字列から `ProjectTemplateType` をパースします。
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "default" => Ok(Self::Default),
            "rust" => Ok(Self::Rust),
            "python" => Ok(Self::Python),
            "dotnet" => Ok(Self::Dotnet),
            "clang" | "cpp" => Ok(Self::CLang),
            _ => Err(format!("Unavailable Template: '{}'", s)),
        }
    }
}

/// 新しいプロジェクト作成のためのパラメータを保持します。
#[derive(Default)] // Default を追加して、ProjectParams のデフォルト実装を容易にする
pub struct ProjectParams {
    pub project_name: String,
    pub project_template: ProjectTemplateType,
    pub author: AuthorAboutData,
}

impl Display for ProjectParams {
    /// `ProjectParams` の内容を整形して表示します。
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}: {}", "Project".bold(), self.project_name)?;
        writeln!(f, "{}: {}", "Template".bold(), self.project_template)?;
        writeln!(
            f,
            "{}: {} <{}>", // Author 情報も表示に追加
            "Author".bold(),
            self.author.name,
            self.author.email
        )
    }
}
impl Display for ProjectTemplateType {
    /// `ProjectTemplateType` の内容を整形して表示します。
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let template_str = match self {
            Self::Default => "default",
            Self::Rust => "rust",
            Self::Python => "python",
            Self::Dotnet => "dotnet",
            Self::CLang => "clang",
        };
        write!(f, "{}", template_str)
    }
}
/// プロジェクト作成中に発生する可能性のあるエラーを定義します。
#[derive(Debug, Error)]
pub enum ProjectCreationError {
    /// YAMLのシリアライズまたはデシリアライズエラー。
    #[error("YAML serialization/deserialization error: {0}")]
    Yaml(#[from] serde_yaml::Error),
    /// I/O関連のエラー。
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
    /// テンプレート固有の作成エラー。
    #[error("Template creation error: {0}")]
    Template(String), // String でエラーメッセージを保持
}

/// 新しいプロジェクトを初期化し、必要なファイルを生成します。
///
/// この関数は、`project_data.yaml` ファイルを作成し、選択されたテンプレートに基づいて
/// その他のプロジェクト構造を生成します。
///
/// # 引数
/// * `params`: 作成するプロジェクトのパラメータ (`ProjectParams` への参照)。
///
/// # 戻り値
/// プロジェクト作成の成否を示す `Result<(), ProjectCreationError>`。
pub fn create(params: &ProjectParams) -> Result<(), ProjectCreationError> {
    // PackageData の初期化と設定
    let mut project_data = PackageData::default();
    project_data.about.package.name = params.project_name.clone(); // to_string() は不要、clone() で十分
    project_data.about.author = params.author.clone();

    // テンプレートに基づくファイル生成
    let project_data = match params.project_template {
        ProjectTemplateType::Default => templates::default(project_data)
            .map_err(|e| ProjectCreationError::Template(e.to_string())),
        ProjectTemplateType::Rust => templates::rust(project_data)
            .map_err(|e| ProjectCreationError::Template(e.to_string())),
        ProjectTemplateType::Python => templates::python(project_data)
            .map_err(|e| ProjectCreationError::Template(e.to_string())),
        ProjectTemplateType::Dotnet => templates::dotnet(project_data)
            .map_err(|e| ProjectCreationError::Template(e.to_string())),
            ProjectTemplateType::CLang => templates::clang(project_data)
            .map_err(|e| ProjectCreationError::Template(e.to_string())),
    }?; // ここで ? 演算子を使用し、エラーを自動伝播

    let project_data_filename = "ipak/project.yaml";
    let data = serde_yaml::to_string(&project_data)?; // YamlError を自動変換

    // file_creation は Result<Result<(), io::Error>, ...> を返す可能性があるので注意
    // ここでは file_creation が直接 io::Result を返すことを想定
    file_creation(project_data_filename, &data)
        .map_err(ProjectCreationError::Io)?; // io::Error を ProjectCreationError::Io にマップ

    Ok(())
}
