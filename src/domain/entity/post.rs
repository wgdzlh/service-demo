use async_trait::async_trait;
use sea_orm::{entity::prelude::*, Set};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::app::utils;

use super::DateTimeTZ;

/// Post to publish
#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel, Serialize, Deserialize, ToSchema)]
#[schema(as = Post)]
#[sea_orm(table_name = "posts")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[schema(read_only, example = 1)]
    pub id: i32,
    #[schema(example = "Hello World")]
    pub title: String,
    #[schema(example = "something to write")]
    pub content: String,
    #[schema(read_only)]
    pub views: i32,
    #[schema(read_only, value_type = String)]
    #[serde(with = "utils::mtime")]
    pub created_at: DateTimeTZ,
    #[schema(read_only, value_type = String)]
    #[serde(with = "utils::mtime")]
    pub updated_at: DateTimeTZ,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

#[async_trait]
impl ActiveModelBehavior for ActiveModel {
    /// Will be triggered before insert / update
    async fn before_save<C>(mut self, _db: &C, insert: bool) -> Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        if insert {
            self.created_at = Set(utils::get_current_time());
            self.updated_at = self.created_at.clone();
        } else if self.title.is_set() || self.content.is_set() {
            self.updated_at = Set(utils::get_current_time());
        }

        Ok(self)
    }
}
