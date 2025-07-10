use super::color::colorize::*;
use std::{fmt, io};

pub struct Error {
    kind: ErrorKind,
    message: String,
}
#[derive(Default)]
pub enum ErrorKind {
    #[default]
    Other,
    Io(io::ErrorKind),
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Other => write!(f, "Other"),
            Self::Io(io_errorkind) => write!(f, "IO-{}", io_errorkind),
        }
    }
}
impl From<&str> for Error {
    fn from(value: &str) -> Self {
        Error::other(value.into())
    }
}
impl From<String> for Error {
    fn from(value: String) -> Self {
        Error::other(value)
    }
}
impl From<io::ErrorKind> for Error {
    fn from(value: io::ErrorKind) -> Self {
        Error::new(ErrorKind::Io(value), "".into())
    }
}
impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Error::new(ErrorKind::Io(value.kind()), value.to_string())
    }
}

impl Error {
    pub fn other(message: String) -> Self {
        Self { kind: ErrorKind::Other, message }
    }
    pub fn new(kind: ErrorKind, message: String) -> Self {
        Self { kind, message }
    }
    fn display_for(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.message.is_empty() {
            write!(f, "  {}", self.kind.to_string().cyan().bold())?;
        } else {
            write!(f, "  {}: {}", "Kind".bold().cyan(), self.kind)?;
            let formatted_message = {
                let split: Vec<String> = self
                    .message
                    .split("\n")
                    .map(|line| format!("    {}", line))
                    .collect();
                split.join("\n")
            };
            write!(
                f,
                "  {}:|\n{}",
                "Message".bold().green(),
                formatted_message
            )?;
        }
        Ok(())
    }
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}:", "Error".red().bold())?;
        self.display_for(f)
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f)?;
        self.display_for(f)
    }
}
