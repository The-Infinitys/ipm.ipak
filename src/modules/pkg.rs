//! `ipak` パッケージ管理モジュールは、パッケージメタデータ、依存関係、インストールモード、
//! およびコマンドライン引数の処理に必要なコアデータ構造と操作を定義します。
//!
//! このモジュールは以下の機能を提供します：
//! - パッケージのインストール（ローカルおよびグローバルモード）
//! - 依存関係と競合の管理
//! - パッケージメタデータの処理
//! - コマンドラインインターフェース操作

use super::version::{Version, VersionRange};
use crate::utils::args::PkgCommands;
use crate::utils::color::colorize::*;
use crate::utils::error::Error;
use crate::utils::{
    generate_email_address,
    shell::{markdown, username},
};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

// モジュール宣言
pub mod depend;
pub mod install;
pub mod list;
pub mod metadata;
pub mod purge;
pub mod remove;

/// パッケージのインストールモードを定義する列挙型。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum Mode {
    /// ローカルインストールモード。現在のプロジェクトに限定されます。
    Local,
    /// グローバルインストールモード。システム全体に適用されます。
    Global,
    /// デフォルトモード。ローカルとグローバルの両方を考慮します。
    #[default]
    Any,
}

impl Display for Mode {
    /// インストールモードを文字列としてフォーマットします。
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Local => write!(f, "local"),
            Self::Global => write!(f, "global"),
            Self::Any => write!(f, "any (local & global)"),
        }
    }
}

/// パッケージメタデータ全体を表し、作者、アーキテクチャ、依存関係を含みます。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct PackageData {
    /// パッケージおよび作者情報
    pub about: AboutData,
    /// サポートされるアーキテクチャ（空の場合はすべてのアーキテクチャを意味します）
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub architecture: Vec<String>,
    /// インストールモード
    pub mode: Mode,
    /// 依存関係および関連情報
    #[serde(skip_serializing_if = "RelationData::is_empty")]
    pub relation: RelationData,
}

/// 作者およびパッケージ固有のメタデータを含みます。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct AboutData {
    /// 作者の詳細情報
    pub author: AuthorAboutData,
    /// パッケージの詳細情報
    pub package: PackageAboutData,
}

/// 作者の名前とメールアドレスを格納します。
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct AuthorAboutData {
    /// 作者の名前
    pub name: String,
    /// 作者のメールアドレス
    pub email: String,
}

/// パッケージのメタデータ（名前、バージョン、説明）を含みます。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PackageAboutData {
    /// パッケージ名
    pub name: String,
    /// パッケージのバージョン
    pub version: Version,
    /// パッケージの説明（オプション）
    #[serde(skip_serializing_if = "String::is_empty")]
    pub description: String,
}

/// パッケージの依存関係や競合を含む関係を管理します。
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct RelationData {
    /// 必須の依存関係（ORグループ）
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub depend: Vec<Vec<PackageRange>>,
    /// 必要なコマンドラインツール
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub depend_cmds: Vec<String>,
    /// 推奨されるオプションの依存関係
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub suggests: Vec<Vec<PackageRange>>,
    /// 推奨される依存関係
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub recommends: Vec<Vec<PackageRange>>,
    /// 競合するパッケージ
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub conflicts: Vec<PackageRange>,
    /// 仮想パッケージの提供
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub virtuals: Vec<PackageVersion>,
    /// このパッケージが提供するコマンド
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub provide_cmds: Vec<String>,
}

/// バージョンの制約を持つパッケージ依存関係を表します。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PackageRange {
    /// パッケージ名
    pub name: String,
    /// バージョンの制約
    pub range: VersionRange,
}

/// 特定のバージョンのパッケージを表します。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct PackageVersion {
    /// パッケージ名
    pub name: String,
    /// 特定のバージョン
    pub version: Version,
}

// Displayトレイトの実装
impl Display for PackageData {
    /// パッケージデータをフォーマットして表示します。
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "{} {}",
            "パッケージ:".bold(),
            self.about.package.name.cyan()
        )?;
        writeln!(
            f,
            "{} {}",
            "バージョン:".bold(),
            self.about.package.version
        )?;

        if !self.about.package.description.is_empty() {
            writeln!(
                f,
                "{} {}",
                "説明:".bold(),
                self.about.package.description
            )?;
        }

        writeln!(
            f,
            "{} {} <{}>",
            "作者:".bold(),
            self.about.author.name.trim(),
            self.about.author.email
        )?;

        writeln!(
            f,
            "{} {}",
            "アーキテクチャ:".bold(),
            if self.architecture.is_empty() {
                "任意".italic()
            } else {
                self.architecture.join(", ").italic()
            }
        )?;

        writeln!(f, "{} {}", "インストールモード:".bold(), self.mode)?;
        write!(f, "{}", self.relation)
    }
}

impl Display for AboutData {
    /// メタデータをフォーマットして表示します。
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{} {}", "作者:".bold(), self.author)?;
        writeln!(f, "{} {}", "パッケージ:".bold(), self.package)
    }
}

impl Display for AuthorAboutData {
    /// 作者情報をフォーマットして表示します。
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} <{}>", self.name, self.email)
    }
}

impl Display for PackageAboutData {
    /// パッケージ情報をフォーマットして表示します。
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.name.cyan(), self.version)?;
        if !self.description.is_empty() {
            write!(f, "\n  {}", markdown(&self.description))?;
        }
        Ok(())
    }
}

impl Display for RelationData {
    /// 依存関係情報をフォーマットして表示します。
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn format_group(
            group: &[PackageRange],
            color: fn(&str) -> String,
        ) -> String {
            if group.len() == 1 {
                format!("{}", color(&group[0].to_string()))
            } else {
                let alts: Vec<String> =
                    group.iter().map(|d| d.to_string()).collect();
                format!("({})", color(&alts.join(" | ")))
            }
        }

        // 依存関係を表示します。
        if !self.depend.is_empty() {
            writeln!(f, "\n{}", "依存関係:".bold())?;
            for group in &self.depend {
                writeln!(f, "  - {}", format_group(group, |s| s.green()))?;
            }
        }

        // 必要なコマンドを表示します。
        if !self.depend_cmds.is_empty() {
            writeln!(f, "\n{}", "必要なコマンド:".bold())?;
            for cmd in &self.depend_cmds {
                writeln!(f, "  - {}", cmd.green())?;
            }
        }

        // 推奨される依存関係（オプション）を表示します。
        if !self.suggests.is_empty() {
            writeln!(f, "\n{}", "推奨（オプション）:".bold())?;
            for group in &self.suggests {
                writeln!(
                    f,
                    "  - {}",
                    format_group(group, |s| s.yellow())
                )?;
            }
        }

        // 推奨される依存関係を表示します。
        if !self.recommends.is_empty() {
            writeln!(f, "\n{}", "推奨:".bold())?;
            for group in &self.recommends {
                writeln!(f, "  - {}", format_group(group, |s| s.blue()))?;
            }
        }

        // 競合するパッケージを表示します。
        if !self.conflicts.is_empty() {
            writeln!(f, "\n{}", "競合:".bold())?;
            for conflict in &self.conflicts {
                writeln!(f, "  - {}", conflict.to_string().red())?;
            }
        }

        // 仮想パッケージを表示します。
        if !self.virtuals.is_empty() {
            writeln!(f, "\n{}", "仮想パッケージ:".bold())?;
            for virtual_pkg in &self.virtuals {
                writeln!(f, "  - {}", virtual_pkg.to_string().magenta())?;
            }
        }

        // 提供するコマンドを表示します。
        if !self.provide_cmds.is_empty() {
            writeln!(f, "\n{}", "提供するコマンド:".bold())?;
            for cmd in &self.provide_cmds {
                writeln!(f, "  - {}", cmd.green())?;
            }
        }
        Ok(())
    }
}

impl Display for PackageRange {
    /// パッケージ依存関係をフォーマットして表示します。
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.name, self.range)
    }
}

impl Display for PackageVersion {
    /// パッケージバージョンをフォーマットして表示します。
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.name, self.version)
    }
}

// デフォルト実装
impl Default for AuthorAboutData {
    /// 作者情報のデフォルト値を生成します。
    fn default() -> Self {
        Self { name: username(), email: generate_email_address() }
    }
}

impl Default for PackageAboutData {
    /// パッケージ情報のデフォルト値を生成します。
    fn default() -> Self {
        Self {
            name: "default-package".to_string(),
            version: Version::default(),
            description: String::new(),
        }
    }
}

impl Default for PackageRange {
    /// パッケージ依存関係のデフォルト値を生成します。
    fn default() -> Self {
        Self {
            name: "default-dependency".to_string(),
            range: VersionRange::default(),
        }
    }
}

impl Default for PackageVersion {
    /// パッケージバージョンのデフォルト値を生成します。
    fn default() -> Self {
        Self {
            name: "default-version".to_string(),
            version: Version::default(),
        }
    }
}

impl RelationData {
    /// すべての依存関係フィールドが空かどうかを確認します。
    pub fn is_empty(&self) -> bool {
        self.depend.is_empty()
            && self.depend_cmds.is_empty()
            && self.suggests.is_empty()
            && self.recommends.is_empty()
            && self.conflicts.is_empty()
            && self.virtuals.is_empty()
            && self.provide_cmds.is_empty()
    }
}

/// コマンドライン引数に基づいてパッケージ関連のコマンドを処理します。
///
/// # 引数
/// * `args` - 処理するパッケージコマンド
///
/// # エラー
/// コマンドの処理中にエラーが発生した場合、`Error`を返します。
pub fn pkg(args: PkgCommands) -> Result<(), Error> {
    match args {
        PkgCommands::Install { file_path, local, global } => {
            install::install(file_path, (local, global).into())
        }
        PkgCommands::Remove { package_name, local, global } => {
            remove::remove(package_name, (local, global).into())
        }
        PkgCommands::Purge { package_name, local, global } => {
            purge::purge(package_name, (local, global).into())
        }
        PkgCommands::List { local, global } => {
            list::list((local, global).into())
        }
        PkgCommands::MetaData { package_path } => {
            metadata::metadata(package_path)
        }
    }
}

// テストモジュール
#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    /// 依存関係や新しいフィールドを含む表示テスト
    #[test]
    fn test_display_with_relations_and_new_fields() {
        let mut data = PackageData::default();
        data.about.author = AuthorAboutData {
            name: "Test Author".to_string(),
            email: "test@example.com".to_string(),
        };
        data.about.package = PackageAboutData {
            name: "my-package".to_string(),
            version: Version::default(),
            description:
                "これはデモンストレーション用のテストパッケージです。"
                    .to_string(),
        };
        data.architecture =
            vec!["x86_64".to_string(), "aarch64".to_string()];
        data.mode = Mode::Global;

        // テスト依存関係の追加
        data.relation.depend.push(vec![PackageRange {
            name: "dep-a".to_string(),
            range: VersionRange::from_str(">= 1.0, < 2.0").unwrap(),
        }]);
        data.relation.depend.push(vec![
            PackageRange {
                name: "dep-b".to_string(),
                range: VersionRange::from_str("= 2.0.0").unwrap(),
            },
            PackageRange {
                name: "dep-c".to_string(),
                range: VersionRange::from_str("> 1.5.0").unwrap(),
            },
        ]);

        // テスト推奨（オプション）の追加
        data.relation.suggests.push(vec![PackageRange {
            name: "suggest-x".to_string(),
            range: VersionRange::from_str("= 3.0").unwrap(),
        }]);

        // テスト推奨の追加
        data.relation.recommends.push(vec![
            PackageRange {
                name: "rec-y".to_string(),
                range: VersionRange::from_str("< 4.0.0").unwrap(),
            },
            PackageRange {
                name: "rec-z".to_string(),
                range: VersionRange::from_str("= 4.1.0").unwrap(),
            },
        ]);

        // テスト競合の追加
        data.relation.conflicts.push(PackageRange {
            name: "old-package".to_string(),
            range: VersionRange::from_str("0.9.0").unwrap(),
        });

        // テスト仮想パッケージの追加
        data.relation.virtuals.push(PackageVersion {
            name: "my-virtual-pkg".to_string(),
            version: Version::from_str("1.0.0").unwrap(),
        });

        // テストコマンドの追加
        data.relation.provide_cmds.extend(vec![
            "my-command".to_string(),
            "another-command".to_string(),
        ]);
        data.relation
            .depend_cmds
            .extend(vec!["git".to_string(), "make".to_string()]);

        println!("\n--- 依存関係と新しいフィールドの表示テスト ---");
        println!("{}", data);
        assert_eq!(
            data.architecture,
            vec!["x86_64".to_string(), "aarch64".to_string()]
        );
        assert_eq!(data.mode, Mode::Global);
    }
    /// 作者情報の表示テスト
    #[test]
    fn test_display_author() {
        let author = AuthorAboutData {
            name: "Test Author".to_string(),
            email: "test@example.com".to_string(),
        };
        println!("\n--- 作者表示テスト ---");
        println!("{}", author);
    }

    /// パッケージ情報の表示テスト
    #[test]
    fn test_display_package() {
        let package = PackageAboutData {
            name: "test-package".to_string(),
            version: Version::default(),
            description: "テストパッケージの簡単な説明。".to_string(),
        };
        println!("\n--- パッケージ表示テスト ---");
        println!("{}", package);

        let package_no_desc = PackageAboutData {
            name: "test-package-no-desc".to_string(),
            version: Version::default(),
            description: String::new(),
        };
        println!("\n--- パッケージ表示テスト（説明なし） ---");
        println!("{}", package_no_desc);
    }

    /// 依存関係情報の表示テスト
    #[test]
    fn test_display_relation() {
        let mut relation = RelationData::default();
        relation.depend.push(vec![PackageRange {
            name: "dep-a".to_string(),
            range: VersionRange::from_str(">= 1.0").unwrap(),
        }]);
        relation.suggests.push(vec![PackageRange {
            name: "suggest-x".to_string(),
            range: VersionRange::from_str("= 3.0").unwrap(),
        }]);
        relation.conflicts.push(PackageRange {
            name: "conflicting-pkg".to_string(),
            range: VersionRange::from_str("< 1.0").unwrap(),
        });
        println!("\n--- 依存関係表示テスト ---");
        println!("{}", relation);
    }

    /// パッケージ依存関係の表示テスト
    #[test]
    fn test_display_package_range() {
        let range = PackageRange {
            name: "test-dep".to_string(),
            range: VersionRange::from_str(">= 1.0").unwrap(),
        };
        println!("\n--- パッケージ範囲表示テスト ---");
        println!("{}", range);
    }

    /// パッケージバージョンの表示テスト
    #[test]
    fn test_display_package_version() {
        let version = PackageVersion {
            name: "test-version".to_string(),
            version: Version::default(),
        };
        println!("\n--- パッケージバージョン表示テスト ---");
        println!("{}", version);
    }

    /// インストールモードの表示テスト
    #[test]
    fn test_mode_display() {
        println!("\n--- モード表示テスト ---");
        println!("ローカル: {}", Mode::Local);
        println!("グローバル: {}", Mode::Global);
        println!("任意: {}", Mode::Any);
    }
}
