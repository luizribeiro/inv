use std::fmt::{Display, Formatter};

pub type Result<T> = std::result::Result<T, AppError>;

#[derive(Debug)]
pub enum AppError {
    NotImplemented(&'static str),
    InvalidUrl {
        source: String,
        reason: &'static str,
    },
    Validation(String),
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
        }
    }
}

impl std::error::Error for AppError {}
