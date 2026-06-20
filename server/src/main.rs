mod channel_routes;
mod crypto;
mod db;
mod error;
mod models;
mod routes;
mod state;

use anyhow::Context;
use axum::routing::{get, post};
use axum::Router;
use sqlx::sqlite::SqlitePoolOptions;
use std::collections::HashMap;
use std::env;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

use crate::channel_routes::{create_channel, delete_channel, list_channels};
use crate::routes::{
    delete_messages, list_messages, push_message, register_device, stream_channel, sync_messages,
};
use crate::state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let database_url =
        env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://tpush.sqlite".to_owned());
    let database = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .with_context(|| format!("failed to connect database: {database_url}"))?;
    db::migrate(&database).await?;

    let state = AppState {
        database,
        clients: Arc::new(Mutex::new(HashMap::new())),
    };

    let app = Router::new()
        .route("/api/devices/register", post(register_device))
        .route("/api/channels", get(list_channels).post(create_channel))
        .route(
            "/api/channels/:channel",
            axum::routing::delete(delete_channel),
        )
        .route("/api/channels/:channel/stream", get(stream_channel))
        .route("/api/push", post(push_message))
        .route("/api/messages", get(list_messages).delete(delete_messages))
        .route("/api/messages/sync", get(sync_messages))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let address: SocketAddr = env::var("BIND_ADDRESS")
        .unwrap_or_else(|_| "0.0.0.0:3000".to_owned())
        .parse()
        .context("invalid BIND_ADDRESS")?;
    let listener = tokio::net::TcpListener::bind(address).await?;
    tracing::info!("TPush server listening on {address}");
    axum::serve(listener, app).await?;
    Ok(())
}
