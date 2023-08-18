use std::net::{Ipv4Addr, SocketAddr};

use axum::{routing, Router, Server};
use const_format::concatcp;
use hyper::Error;
use tokio::signal;
use tower_http::trace::{DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tracing::Level;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    app::log::info,
    doc::ApiDoc,
    infrastructure::{config::BASE_PATH, persistence::Db},
    interface::handler::{post, todo},
};

pub async fn serve(db: Db) -> Result<(), Error> {
    let todo_handler = Router::new()
        .route(
            "/",
            routing::get(todo::list)
                .post(todo::create)
                .put(todo::edit)
                .delete(todo::delete),
        )
        .route("/:id", routing::put(todo::mark_done))
        .with_state(db.todo.clone());

    let post_handler = Router::new()
        .route(
            "/",
            routing::get(post::list)
                .post(post::create)
                .put(post::edit)
                .delete(post::delete),
        )
        .route("/:id", routing::get(post::get))
        .with_state(db.post.clone());

    let root = Router::new()
        .nest("/todo", todo_handler)
        .nest("/post", post_handler)
        .layer(
            TraceLayer::new_for_http()
                .on_request(DefaultOnRequest::new().level(Level::TRACE))
                .on_response(DefaultOnResponse::new().level(Level::INFO)),
        );

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
