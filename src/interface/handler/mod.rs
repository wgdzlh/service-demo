pub mod post;
pub mod read_xls;
pub mod todo;

use axum::{response::IntoResponse, Json};

use crate::repository::{Error, Result};

use super::resp::{ObjectRes, Response, Void, VoidRes};

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::BadRequest => Json(Response::new_bad(Void {})).into_response(),
            _ => Json(Response::new_err(Void {}, self.to_string())).into_response(),
        }
    }
}

fn ok_resp() -> VoidRes {
    Response::new(Void {})
}

fn to_raw_resp(s: String) -> Result<ObjectRes> {
    Ok(Response::new(serde_json::from_slice(s.as_bytes())?))
}
