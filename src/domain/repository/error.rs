use std::{fmt, io, sync::PoisonError};

/// entity repo errors
#[derive(Debug)]
// #[serde(tag = "type", content = "detail")]
pub enum Error {
    BadRequest,
    IdNotFound { id: i32 },
    DbError(String),
    LockFailed(String),
    IoError(String),
    EmptyRet,
    SubmitTimeout,
    RunSubCmdError(String),
    Other(String),
}

pub type Result<T> = core::result::Result<T, Error>;

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
