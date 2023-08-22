use std::sync::Arc;

use async_trait::async_trait;
use sea_orm::sea_query::Expr;
use sea_orm::*;

use crate::{
    app::log::*,
    entity::{
        post::{ActiveModel, Column, Entity},
        Post,
    },
    repository::{Error, PostNew, PostQuery, PostRepo, PostUpdate, Result},
};

pub type PostStore = Arc<dyn PostRepo + Send + Sync>;

pub(super) fn get_post_store(db: &DbConn) -> PostStore {
    Arc::new(PostRepoImp { db: db.clone() })
}

/// Database Post store
struct PostRepoImp {
    db: DbConn,
}

#[async_trait]
impl PostRepo for PostRepoImp {
    async fn create(&self, item: PostNew) -> Result<i32> {
        info!(?item, "create post");
        let res = ActiveModel {
            title: Set(item.title.to_owned()),
            content: Set(item.content.to_owned()),
            ..Default::default()
        }
        .save(&self.db)
        .await?;
        Ok(match res.id {
            ActiveValue::Set(v) | ActiveValue::Unchanged(v) => v,
            NotSet => 0,
        })
    }

    async fn update(&self, item: PostUpdate) -> Result<()> {
        info!(?item, "update post");
        ActiveModel {
            id: Unchanged(item.id),
            title: item.title.map_or(NotSet, Set),
            content: item.content.map_or(NotSet, Set),
            ..Default::default()
        }
        .update(&self.db)
        .await?;
        Ok(())
    }

    async fn delete(&self, ids: Vec<i32>) -> Result<()> {
        info!(?ids, "delete posts");
        Entity::delete_many()
            .filter(Column::Id.is_in(ids))
            .exec(&self.db)
            .await?;
        Ok(())
    }

    async fn fetch(&self, id: i32) -> Result<Post> {
        info!(?id, "fetch post");
        let res = Entity::find_by_id(id).one(&self.db).await?;
        match res {
            Some(v) => {
                Entity::update_many()
                    .col_expr(Column::Views, Expr::col(Column::Views).add(1))
                    .filter(Column::Id.eq(id))
                    .exec(&self.db)
                    .await?;
                // self.db
                //     .execute(Statement::from_sql_and_values(
                //         DbBackend::Postgres,
                //         "UPDATE posts SET views = views + 1 WHERE id = $1",
                //         [id.into()],
                //     ))
                //     .await?;
                Ok(v)
            }
            None => Err(Error::IdNotFound { id }),
        }
    }

    async fn query(&self, params: PostQuery) -> Result<(Vec<Post>, u64)> {
        info!(?params, "query posts");
        let page = params.page.unwrap_or(1);
        let size = params.size.unwrap_or(10);

        let mut cur = Entity::find();
        if let Some(v) = params.title {
            cur = cur.filter(Column::Title.contains(v));
        }
        if let Some(v) = params.content {
            cur = cur.filter(Column::Content.contains(v));
        }

        let paginator = cur.order_by_desc(Column::Id).paginate(&self.db, size);

        let total = paginator.num_items().await?;
        let res = paginator.fetch_page(page - 1).await?;
        Ok((res, total))
    }
}
