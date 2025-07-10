use super::color::colorize::*;
use std::{fmt, io};

/// アプリケーション全体で利用されるカスタムエラー構造体です。
/// エラーの種類と詳細なメッセージを保持します。
pub struct Error {
    kind: ErrorKind,
    message: String,
}

/// エラーの種類を定義する列挙型です。
#[derive(Default)]
pub enum ErrorKind {
    /// その他の一般的なエラー。
    #[default]
    Other,
    /// I/O関連のエラー。
    Io(io::ErrorKind),
}

impl fmt::Display for ErrorKind {
    /// `ErrorKind`を文字列としてフォーマットします。
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Other => write!(f, "Other"),
            Self::Io(io_errorkind) => write!(f, "IO-{}", io_errorkind),
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
        Error::new(ErrorKind::Io(value), "".into())
    }
}

impl From<io::Error> for Error {
    /// `io::Error`から`Error`を生成します。
    fn from(value: io::Error) -> Self {
        Error::new(ErrorKind::Io(value.kind()), value.to_string())
    }
}

impl Error {
    /// その他の種類のエラーを生成します。
    ///
    /// # Arguments
    /// * `message` - エラーメッセージ
    pub fn other(message: String) -> Self {
        Self { kind: ErrorKind::Other, message }
    }

    /// 指定された種類とメッセージで新しいエラーを生成します。
    ///
    /// # Arguments
    /// * `kind` - エラーの種類
    /// * `message` - エラーメッセージ
    pub fn new(kind: ErrorKind, message: String) -> Self {
        Self { kind, message }
    }

    /// エラー情報をフォーマットして表示します。
    fn display_for(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.message.is_empty() {
            write!(f, "  {}", self.kind.to_string().cyan().bold())?;
        } else {
            write!(f, "  {}: {}", "Kind".bold().cyan(), self.kind)?;
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
