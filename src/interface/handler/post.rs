use axum::{
    extract::{Query, State},
    Json,
};

use crate::{
    app::utils,
    infrastructure::persistence::PostStore,
    interface::resp::*,
    repository::{Error, PostDelete, PostNew, PostQuery, PostUpdate, Result},
};

use super::ok_resp;

/// Query Post items
///
/// Query Post items from in-memory storage.
#[utoipa::path(
        get,
        path = "/post",
        params(
            PostQuery
        ),
        responses(
            (status = 200, description = "List matching posts by query", body = PostListRes)
        )
    )]
pub async fn list(
    store: State<PostStore>,
    Query(params): Query<PostQuery>,
) -> Result<Json<PostListRes>> {
    let posts = store.query(params).await?;
    Ok(Json(Response::new(posts.into())))
}

/// Create new Post
///
/// Tries to create a new Post item to in-memory storage or fails with 409 conflict if already exists.
#[utoipa::path(
        post,
        path = "/post",
        request_body = PostNew,
        responses(
            (status = 200, description = "Post item created successfully", body = IdRes)
        )
    )]
pub async fn create(store: State<PostStore>, Json(post): Json<PostNew>) -> Result<Json<IdRes>> {
    if post.title.is_empty() {
        return Err(Error::BadRequest);
    }
    let new_id = store.create(post).await?;
    Ok(Json(Response::new(IdData { id: new_id })))
}

/// Edit Post item value by id
///
/// Edit Post item value by given id.
#[utoipa::path(
        put,
        path = "/post",
        request_body = PostUpdate,
        responses(
            (status = 200, description = "Post marked done successfully", body = VoidRes),
        )
    )]
pub async fn edit(store: State<PostStore>, Json(post): Json<PostUpdate>) -> Result<Json<VoidRes>> {
    store.update(post).await?;
    Ok(Json(ok_resp()))
}

/// Delete Post items by id
///
/// Delete Post items from in-memory storage by ids seperated by comma.
#[utoipa::path(
        delete,
        path = "/post",
        params(
            PostDelete
        ),
        responses(
            (status = 200, description = "Post marked done successfully", body = VoidRes)
        )
    )]
pub async fn delete(
    store: State<PostStore>,
    Query(params): Query<PostDelete>,
) -> Result<Json<VoidRes>> {
    store.delete(utils::get_ids_from_str(&params.ids)).await?;
    Ok(Json(ok_resp()))
}
