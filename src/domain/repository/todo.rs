use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

use crate::entity::Todo;

use super::Result;

pub trait TodoRepo {
    fn create(&self, item: Todo) -> Result<i32>;
    fn update(&self, item: TodoUpdate) -> Result<()>;
    fn delete(&self, ids: Vec<i32>) -> Result<()>;
    fn fetch(&self, id: i32) -> Result<Todo>;
    fn query(&self, params: TodoQuery) -> Result<Vec<Todo>>;
}

/// Todo update params
#[derive(Debug, Deserialize, ToSchema)]
pub struct TodoUpdate {
    pub id: i32,
    pub value: Option<String>,
    pub done: Option<bool>,
}

/// Todo search query
#[derive(Debug, Deserialize, IntoParams)]
pub struct TodoQuery {
    /// Search by value. Search is case insensitive.
    pub value: Option<String>,
    /// Search by `done` status.
    pub done: Option<bool>,
}

#[derive(Deserialize, IntoParams)]
pub struct TodoDelete {
    #[param(example = "1,2,3")]
    pub ids: String,
}
