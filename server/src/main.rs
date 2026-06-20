mod auth;
mod channel_routes;
mod config;
mod crypto;
mod db;
mod error;
mod models;
mod panel_assets;
mod routes;
mod state;

use anyhow::Context;
use axum::body::Body;
use axum::extract::Request;
use axum::http::{header, StatusCode};
use axum::response::Response;
use axum::routing::{get, post};
use axum::Router;
use sqlx::sqlite::SqlitePoolOptions;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use crate::auth::{auth_middleware, login};
use crate::channel_routes::{create_channel, delete_channel, list_channels};
use crate::config::AppConfig;
use crate::panel_assets::PanelAssets;
use crate::routes::{
    delete_messages, list_messages, push_message, register_device, stream_channel, sync_messages,
};
use crate::state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let config = AppConfig::load()?;
    tracing::info!("loaded config, bind_address={}", config.server.bind_address);

    let database = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(
            &std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "sqlite://tpush.sqlite".to_owned()),
        )
        .await
        .context("failed to connect database")?;
    db::migrate(&database).await?;

    let state = AppState {
        database,
        clients: Arc::new(Mutex::new(HashMap::new())),
        config: config.clone(),
    };

    // Public API — no auth required (client + third-party calls)
    let public_api = Router::new()
        .route("/api/devices/register", post(register_device))
        .route("/api/channels/:channel/stream", get(stream_channel))
        .route("/api/messages/sync", get(sync_messages))
        .route("/api/push", post(push_message))
        .route("/api/messages", get(list_messages).delete(delete_messages));

    // Protected admin API — JWT auth required (channel management only)
    let admin_api = Router::new()
        .route("/api/channels", get(list_channels).post(create_channel))
        .route(
            "/api/channels/:channel",
            axum::routing::delete(delete_channel),
        )
        .route_layer(axum::middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    let app = Router::new()
        .merge(public_api)
        .merge(admin_api)
        .route("/api/admin/login", post(login))
        .fallback_service(get(serve_embedded))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let address: SocketAddr = config
        .server
        .bind_address
        .parse()
        .context("invalid bind_address in config")?;
    let listener = tokio::net::TcpListener::bind(address).await?;
    tracing::info!("TPush server listening on {address}");
    axum::serve(listener, app).await?;
    Ok(())
}

async fn serve_embedded(req: Request) -> Response {
    let path = req.uri().path().trim_start_matches('/');
    let path = if path.is_empty() { "index.html" } else { path };

    if let Some(file) = PanelAssets::get(path) {
        let mime = mime_guess::from_path(path).first_or_octet_stream();
        return Response::builder()
            .header(header::CONTENT_TYPE, mime.as_ref())
            .body(Body::from(file.data))
            .unwrap();
    }

    // SPA fallback: any unmatched route serves index.html
    if let Some(index) = PanelAssets::get("index.html") {
        return Response::builder()
            .header(header::CONTENT_TYPE, "text/html; charset=utf-8")
            .body(Body::from(index.data))
            .unwrap();
    }

    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Body::from("not found"))
        .unwrap()
}
