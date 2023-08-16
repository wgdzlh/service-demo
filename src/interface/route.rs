use std::net::{Ipv4Addr, SocketAddr};

use axum::{routing, Router, Server};
use const_format::concatcp;
use hyper::Error;
use tokio::signal;
use tracing::info;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    doc::ApiDoc,
    infrastructure::{config::BASE_PATH, persistence::Db},
    interface::handler::{post, todo},
};

pub async fn serve(db: Db) -> Result<(), Error> {
    let todo_repo = Router::new()
        .route(
            "/",
            routing::get(todo::list)
                .post(todo::create)
                .put(todo::edit)
                .delete(todo::delete),
        )
        .route("/:id", routing::put(todo::mark_done))
        .with_state(db.todo.clone());

    let post_repo = Router::new()
        .route(
            "/",
            routing::get(post::list)
                .post(post::create)
                .put(post::edit)
                .delete(post::delete),
        )
        .with_state(db.post.clone());

    let root = Router::new()
        .nest("/todo", todo_repo)
        .nest("/post", post_repo);

    let app = Router::new()
        .merge(
            SwaggerUi::new(concatcp!(BASE_PATH, "/swagger-ui"))
                .url("/api-docs/openapi.json", ApiDoc::openapi()),
        )
        .nest(BASE_PATH, root);

    let address = SocketAddr::from((Ipv4Addr::UNSPECIFIED, 8080));
    Server::bind(&address)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    // #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    // #[cfg(not(unix))]
    // let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("signal received, starting graceful shutdown");
}
