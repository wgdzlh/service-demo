use axum::{
    extract::{Path, Query, State},
    Json,
};

use crate::{
    app::utils,
    entity::Todo,
    infrastructure::persistence::TodoStore,
    interface::resp::*,
    repository::{Error, Result, TodoDelete, TodoQuery, TodoUpdate},
};

use super::ok_resp;

/// Query Todo items
///
/// Query Todo items from in-memory storage.
#[utoipa::path(
        get,
        path = "/todo",
        params(
            TodoQuery
        ),
        responses(
            (status = 200, description = "List matching todos by query", body = TodoListRes)
        )
    )]
pub async fn list(
    store: State<TodoStore>,
    Query(params): Query<TodoQuery>,
) -> Result<Json<TodoListRes>> {
    let todos = store.query(params)?;
    Ok(Json(Response::new(todos)))
}

/// Create new Todo
///
/// Try to create a new Todo item to in-memory storage.
#[utoipa::path(
        post,
        path = "/todo",
        request_body = Todo,
        responses(
            (status = 200, description = "Todo item created successfully", body = IdRes)
        )
    )]
pub async fn create(store: State<TodoStore>, Json(todo): Json<Todo>) -> Result<Json<IdRes>> {
    if todo.value.is_empty() {
        return Err(Error::BadRequest);
    }
    let new_id = store.create(todo)?;
    Ok(Json(Response::new(IdData { id: new_id })))
}

/// Mark Todo item done by id
///
/// Mark Todo item done by given id.
#[utoipa::path(
        put,
        path = "/todo/{id}",
        params(
            ("id" = i32, Path, description = "Todo item id")
        ),
        responses(
            (status = 200, description = "Todo marked done successfully", body = VoidRes)
        )
    )]
pub async fn mark_done(store: State<TodoStore>, Path(id): Path<i32>) -> Result<Json<VoidRes>> {
    store.update(TodoUpdate {
        id,
        value: None,
        done: Some(true),
    })?;
    Ok(Json(ok_resp()))
}

/// Edit Todo item value by id
///
/// Edit Todo item value by given id.
#[utoipa::path(
        put,
        path = "/todo",
        request_body = TodoUpdate,
        responses(
            (status = 200, description = "Todo marked done successfully", body = VoidRes)
        )
    )]
pub async fn edit(store: State<TodoStore>, Json(todo): Json<TodoUpdate>) -> Result<Json<VoidRes>> {
    store.update(todo)?;
    Ok(Json(ok_resp()))
}

/// Delete Todo items by id
///
/// Delete Todo items from in-memory storage by comma-separated ids.
#[utoipa::path(
        delete,
        path = "/todo",
        params(
            TodoDelete
        ),
        responses(
            (status = 200, description = "Todo marked done successfully", body = VoidRes)
        )
    )]
pub async fn delete(
    store: State<TodoStore>,
    Query(params): Query<TodoDelete>,
) -> Result<Json<VoidRes>> {
    store.delete(utils::get_ids_from_str(&params.ids))?;
    Ok(Json(ok_resp()))
}
