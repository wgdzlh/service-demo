use std::{fmt, io, string, sync};

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
    EncodingError(String),
    EmptyRet,
    SubmitTimeout,
    RunSubCmdError(String),
    JsonParseError(String),
    WorkerQueueError(String),
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

impl<T> From<sync::PoisonError<T>> for Error {
    fn from(value: sync::PoisonError<T>) -> Self {
        Self::LockFailed(value.to_string())
    }
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::IoError(value.to_string())
    }
}

impl From<string::FromUtf8Error> for Error {
    fn from(value: string::FromUtf8Error) -> Self {
        Self::EncodingError(value.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Self::JsonParseError(value.to_string())
    }
}

impl From<sea_orm::DbErr> for Error {
    fn from(value: sea_orm::DbErr) -> Self {
        Self::DbError(value.to_string())
    }
}

impl From<MultipartError> for Error {
    fn from(value: MultipartError) -> Self {
        Self::BadMultipart(value.to_string())
    }
}

impl<T> From<flume::SendError<T>> for Error {
    fn from(value: flume::SendError<T>) -> Self {
        Self::WorkerQueueError(value.to_string())
    }
}

impl From<flume::RecvError> for Error {
    fn from(value: flume::RecvError) -> Self {
        Self::WorkerQueueError(value.to_string())
    }
}
