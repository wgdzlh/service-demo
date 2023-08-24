use std::{fmt, io, sync::PoisonError};

use axum::extract::multipart::MultipartError;

/// entity repo errors
#[derive(Debug)]
// #[serde(tag = "type", content = "detail")]
pub enum Error {
    BadRequest,
    BadMultipart(String),
    IdNotFound { id: i32 },
    DbError(String),
    LockFailed(String),
    IoError(String),
    EmptyRet,
    SubmitTimeout,
    RunSubCmdError(String),
    JsonParseError(String),
    Other(String),
}

pub type Result<T> = std::result::Result<T, Error>;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for Error {}

impl From<String> for Error {
    fn from(value: String) -> Self {
        Self::Other(value)
    }
}

impl From<&str> for Error {
    fn from(value: &str) -> Self {
        value.to_owned().into()
    }
}

impl<T> From<PoisonError<T>> for Error {
    fn from(value: PoisonError<T>) -> Self {
        Self::LockFailed(value.to_string())
    }
}

impl From<sea_orm::DbErr> for Error {
    fn from(value: sea_orm::DbErr) -> Self {
        Self::DbError(value.to_string())
    }
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::IoError(value.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Self::JsonParseError(value.to_string())
    }
}

impl From<MultipartError> for Error {
    fn from(value: MultipartError) -> Self {
        Self::BadMultipart(value.to_string())
    }
}
