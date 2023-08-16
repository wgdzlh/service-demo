use async_trait::async_trait;
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

use crate::entity::Post;

use super::Result;

#[async_trait]
pub trait PostRepo {
    async fn create(&self, item: PostNew) -> Result<i32>;
    async fn update(&self, item: PostUpdate) -> Result<()>;
    async fn delete(&self, ids: Vec<i32>) -> Result<()>;
    async fn fetch(&self, id: i32) -> Result<Post>;
    async fn query(&self, params: PostQuery) -> Result<(Vec<Post>, u64)>;
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct PostNew {
    pub title: String,
    pub content: String,
}

/// Post update params
#[derive(Debug, Deserialize, ToSchema)]
pub struct PostUpdate {
    pub id: i32,
    pub title: Option<String>,
    pub content: Option<String>,
}

/// Post search query
#[derive(Debug, Deserialize, IntoParams)]
pub struct PostQuery {
    /// Search by title, case insensitive
    pub title: Option<String>,
    /// Search by content, case insensitive
    pub content: Option<String>,
    /// page
    #[param(default = 1)]
    pub page: Option<u64>,
    /// page size
    #[param(default = 10)]
    pub size: Option<u64>,
}

#[derive(Deserialize, IntoParams)]
pub struct PostDelete {
    #[param(example = "1,2,3")]
    pub ids: String,
}
