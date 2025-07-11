use super::color::colorize::*;
use std::{fmt, io};

// InstallError と RemoveError をインポート
use crate::modules::pkg::depend::error::{InstallError, RemoveError};

/// アプリケーション全体で利用されるカスタムエラー構造体です。
/// エラーの種類と詳細なメッセージを保持します。
pub struct Error {
    pub kind: ErrorKind,
    pub message: String,
    // 他のエラータイプをラップするためのフィールドを追加
    // 'static ライフタイム制約を追加
    pub source: Option<Box<dyn std::error::Error + Send + Sync + 'static>>,
}

/// エラーの種類を定義する列挙型です。
#[derive(Default, Clone, Copy)]
pub enum ErrorKind {
    /// その他の一般的なエラー。
    #[default]
    Other,
    /// I/O関連のエラー。
    Io(io::ErrorKind),
    /// パッケージインストール関連のエラー。
    Install,
    /// パッケージ削除関連のエラー。
    Remove,
}

impl fmt::Display for ErrorKind {
    /// `ErrorKind`を文字列としてフォーマットします。
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Other => write!(f, "Other"),
            Self::Io(io_errorkind) => write!(f, "IO-{}", io_errorkind),
            Self::Install => write!(f, "Package Installation Error"),
            Self::Remove => write!(f, "Package Removal Error"),
        }
    }
}

impl From<&str> for Error {
    /// 文字列スライスから`Error`を生成します。
    fn from(value: &str) -> Self {
        Error::other(value.into())
    }
}

impl From<String> for Error {
    /// `String`から`Error`を生成します。
    fn from(value: String) -> Self {
        Error::other(value)
    }
}

impl From<io::ErrorKind> for Error {
    /// `io::ErrorKind`から`Error`を生成します。
    fn from(value: io::ErrorKind) -> Self {
        Error::new(ErrorKind::Io(value), "".into(), None)
    }
}

impl From<io::Error> for Error {
    /// `io::Error`から`Error`を生成します。
    fn from(value: io::Error) -> Self {
        // io::Error は std::error::Error を実装しているので、source に渡せる
        Error::new(
            ErrorKind::Io(value.kind()),
            value.to_string(),
            Some(Box::new(value)),
        )
    }
}

// InstallError から Error への変換を実装
impl From<InstallError> for Error {
    fn from(value: InstallError) -> Self {
        Error::new(
            ErrorKind::Install,
            value.to_string(),
            Some(Box::new(value)),
        )
    }
}

// RemoveError から Error への変換を実装
impl From<RemoveError> for Error {
    fn from(value: RemoveError) -> Self {
        Error::new(
            ErrorKind::Remove,
            value.to_string(),
            Some(Box::new(value)),
        )
    }
}

impl Error {
    /// その他の種類のエラーを生成します。
    ///
    /// # Arguments
    /// * `message` - エラーメッセージ
    pub fn other(message: String) -> Self {
        Self { kind: ErrorKind::Other, message, source: None }
    }

    /// 指定された種類とメッセージで新しいエラーを生成します。
    ///
    /// # Arguments
    /// * `kind` - エラーの種類
    /// * `message` - エラーメッセージ
    /// * `source` - 元のエラー（オプション）
    pub fn new(
        kind: ErrorKind,
        message: String,
        source: Option<Box<dyn std::error::Error + Send + Sync + 'static>>,
    ) -> Self {
        Self { kind, message, source }
    }
    /// エラー情報をフォーマットして表示します。
    fn display_for(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.message.is_empty() && self.source.is_none() {
            write!(f, "  {}", self.kind.to_string().cyan().bold())?;
        } else {
            write!(f, "  {}: {}", "Kind".bold().cyan(), self.kind)?;
            if !self.message.is_empty() {
                let formatted_message = self
                    .message
                    .split("\n")
                    .map(|line| format!("    {}", line))
                    .collect::<Vec<String>>()
                    .join("\n");
                write!(
                    f,
                    "\n  {}:\n{}",
                    "Message".bold().green(),
                    formatted_message
                )?;
            }
            // 元のエラーがあれば表示
            if let Some(source) = &self.source {
                write!(
                    f,
                    "\n  {}:\n    {}",
                    "Source Error".bold().yellow(),
                    source
                )?;
            }
        }
        Ok(())
    }
}

impl fmt::Display for Error {
    /// `Error`をユーザーフレンドリーな形式で表示します。
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}:", "Error".red().bold())?;
        self.display_for(f)
    }
}

impl fmt::Debug for Error {
    /// `Error`をデバッグ形式で表示します。
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f)?;
        self.display_for(f)
    }
}

// std::error::Error トレイトを実装
impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        // ここで型キャストを行います。
        // self.source.as_ref() は Option<&Box<dyn std::error::Error + Send + Sync + 'static>> を返します。
        // map(|s| s.as_ref()) は Option<&(dyn std::error::Error + Send + Sync + 'static)> を返します。
        // これを &(dyn std::error::Error + 'static) にダウンキャストします。
        self.source
            .as_ref()
            .map(|s| s.as_ref() as &(dyn std::error::Error + 'static))
    }
}
