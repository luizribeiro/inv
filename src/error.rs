use std::fmt::{Display, Formatter};
use std::path::PathBuf;

pub type Result<T> = std::result::Result<T, AppError>;

#[derive(Debug)]
pub enum AppError {
    NotImplemented(&'static str),
    InvalidUrl {
        source: String,
        reason: &'static str,
    },
    Validation(String),
    Io {
        path: PathBuf,
        action: &'static str,
        source: std::io::Error,
    },
    JsonParse {
        path: PathBuf,
        source: serde_json::Error,
    },
}

impl Display for AppError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotImplemented(command) => {
                write!(f, "command '{command}' is not implemented yet")
            }
            Self::InvalidUrl { source, reason } => {
                write!(f, "invalid URL '{source}': {reason}")
            }
            Self::Validation(message) => write!(f, "validation error: {message}"),
            Self::Io {
                path,
                action,
                source,
            } => write!(f, "failed to {action} '{}': {source}", path.display()),
            Self::JsonParse { path, source } => {
                write!(
                    f,
                    "failed to parse JSON from '{}': {source}",
                    path.display()
                )
            }
        }
    }
}

impl std::error::Error for AppError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::JsonParse { source, .. } => Some(source),
            _ => None,
        }
    }
}
