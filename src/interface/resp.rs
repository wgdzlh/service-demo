use serde::Serialize;
use utoipa::ToSchema;

use crate::entity::*;

const STATUS_OK: i32 = 200;
const STATUS_BAD: i32 = 400;
const STATUS_ERR: i32 = 500;

const MSG_OK: &str = "ok";
const MSG_BAD: &str = "bad request";
const MSG_ERR: &str = "error";

#[derive(Serialize, ToSchema)]
pub struct Void {}

#[derive(Serialize, ToSchema)]
pub struct IdData {
    pub id: i32,
}

#[derive(Serialize, ToSchema)]
#[aliases(PostList = ListData<Post>)]
pub struct ListData<T> {
    pub list: Vec<T>,
    pub total: u64,
}

#[derive(Serialize, ToSchema)]
#[aliases(VoidRes = Response<Void>, IdRes = Response<IdData>,
     TodoRes = Response<Todo>, TodoListRes = Response<Vec<Todo>>,
     PostRes = Response<Post>, PostListRes = Response<PostList>)]
pub struct Response<T> {
    /// response code: 200 - ok; 400 - bad request; 500 - error
    #[schema(example = 200)]
    code: i32,
    #[schema(example = "ok")]
    msg: String,
    data: T,
}

impl<T: Serialize> Response<T> {
    pub fn new(data: T) -> Self {
        Self {
            code: STATUS_OK,
            msg: MSG_OK.to_owned(),
            data,
        }
    }

    pub fn new_bad(data: T) -> Self {
        Self {
            code: STATUS_BAD,
            msg: MSG_BAD.to_owned(),
            data,
        }
    }

    pub fn new_err(data: T, msg: String) -> Self {
        Self {
            code: STATUS_ERR,
            msg: match msg.len() {
                0 => MSG_ERR.to_owned(),
                _ => msg,
            },
            data,
        }
    }
}

impl<T> From<(Vec<T>, u64)> for ListData<T> {
    fn from(value: (Vec<T>, u64)) -> Self {
        Self {
            list: value.0,
            total: value.1,
        }
    }
}
