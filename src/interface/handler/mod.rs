pub mod post;
pub mod todo;

use axum::{response::IntoResponse, Json};

use crate::repository::Error;

use super::resp::{Response, Void, VoidRes};

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
