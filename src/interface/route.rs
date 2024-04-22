use std::net::Ipv4Addr;

use axum::{extract::DefaultBodyLimit, routing, Router};
use const_format::concatcp;
use tokio::{net::TcpListener, signal};
use tower_http::trace::{DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tracing::Level;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    app::{self, log::*},
    doc::ApiDoc,
    infrastructure::{config, persistence::Db, shell::ChildWorkers},
    interface::handler::{post, todo},
};

use super::handler::read_xls;

const MAX_BODY_SIZE: usize = 1024 * 1024 * 128;

pub async fn serve(db: Db, child_workers: ChildWorkers) -> app::Result<()> {
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

    let read_xls_handler = Router::new()
        .route("/parse", routing::post(read_xls::parse))
        .with_state(child_workers.read_xls);

    let root = Router::new()
        .nest("/todo", todo_handler)
        .nest("/post", post_handler)
        .nest("/xls", read_xls_handler)
        .layer(DefaultBodyLimit::max(MAX_BODY_SIZE))
        .layer(
            TraceLayer::new_for_http()
                .on_request(DefaultOnRequest::new().level(Level::TRACE))
                .on_response(DefaultOnResponse::new().level(Level::INFO)),
        );

    let app = Router::new()
        .merge(
            SwaggerUi::new(concatcp!(config::BASE_PATH, "/swagger-ui"))
                .url("/api-docs/openapi.json", ApiDoc::openapi()),
        )
        .nest(config::BASE_PATH, root);

    let server_conf = config::peek_config()?.server.clone();
    let port = server_conf.port.unwrap_or(config::DEFAULT_PORT);

    let listener = TcpListener::bind((Ipv4Addr::UNSPECIFIED, port)).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
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
