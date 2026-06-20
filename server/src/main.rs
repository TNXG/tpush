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
use axum::Router;
use axum::body::Body;
use axum::extract::Request;
use axum::http::{StatusCode, header};
use axum::response::Response;
use axum::routing::{get, post};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{AssertSqlSafe, Sqlite};
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::net::SocketAddr;
use std::path::Path;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::EnvFilter;

use crate::auth::{auth_middleware, login};
use crate::channel_routes::{create_channel, delete_channel, list_channels};
use crate::config::{AppConfig, server_project_dir};
use crate::panel_assets::PanelAssets;
use crate::routes::{
    delete_messages, list_devices, list_messages, push_message, register_device, stream_channel,
    sync_messages,
};
use crate::state::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .init();

    let config = AppConfig::load()?;
    tracing::info!("loaded config, bind_address={}", config.server.bind_address);

    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        format!(
            "sqlite://{}",
            server_project_dir().join("tpush.sqlite").display()
        )
    });
    ensure_database_file(&database_url)?;
    tracing::info!(
        database = %sqlite_file_path(&database_url).unwrap_or_else(|| database_url.clone()),
        "database configured"
    );
    let database_options = SqliteConnectOptions::from_str(&database_url)
        .with_context(|| format!("invalid DATABASE_URL: {database_url}"))?
        .create_if_missing(true)
        .read_only(false);
    let database = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(database_options)
        .await
        .context("failed to connect database")?;
    ensure_database_writable(&database, &database_url).await?;
    db::migrate(&database).await?;

    let state = AppState {
        database,
        clients: Arc::new(Mutex::new(HashMap::new())),
        config: config.clone(),
    };

    // Public API — no auth required (client + third-party calls)
    let public_api = Router::new()
        .route("/api/devices/register", post(register_device))
        .route("/api/channels/{channel}/stream", get(stream_channel))
        .route("/api/messages/sync", get(sync_messages))
        .route("/api/push", post(push_message))
        .route("/api/messages", get(list_messages).delete(delete_messages));

    // Protected admin API — JWT auth required (channel management only)
    let admin_api = Router::new()
        .route("/api/devices", get(list_devices))
        .route("/api/channels", get(list_channels).post(create_channel))
        .route(
            "/api/channels/{channel}",
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
    tracing::warn!(
        listen = %address,
        panel_url = %format!("http://{address}"),
        "TPush server started successfully"
    );
    axum::serve(listener, app).await?;
    Ok(())
}

async fn ensure_database_writable(
    database: &sqlx::Pool<sqlx::Sqlite>,
    database_url: &str,
) -> anyhow::Result<()> {
    let user_version = sqlx::query_scalar::<_, i64>("PRAGMA user_version")
        .fetch_one(database)
        .await
        .with_context(|| {
            let path = sqlite_file_path(database_url).unwrap_or_else(|| database_url.to_owned());
            format!("failed to read database metadata: {path}")
        })?;
    let mut connection = database.acquire().await.with_context(|| {
        let path = sqlite_file_path(database_url).unwrap_or_else(|| database_url.to_owned());
        format!("failed to acquire database connection: {path}")
    })?;
    sqlx::query("BEGIN IMMEDIATE")
        .execute(&mut *connection)
        .await
        .with_context(|| {
            let path = sqlite_file_path(database_url).unwrap_or_else(|| database_url.to_owned());
            format!("database is not writable: {path}")
        })?;
    let probe_result = sqlx::query::<Sqlite>(AssertSqlSafe(format!(
        "PRAGMA user_version = {user_version}"
    )))
    .execute(&mut *connection)
    .await;
    let rollback_result = sqlx::query("ROLLBACK").execute(&mut *connection).await;
    probe_result.with_context(|| {
        let path = sqlite_file_path(database_url).unwrap_or_else(|| database_url.to_owned());
        format!("database is not writable: {path}")
    })?;
    rollback_result.with_context(|| {
        let path = sqlite_file_path(database_url).unwrap_or_else(|| database_url.to_owned());
        format!("failed to finish database write probe: {path}")
    })?;
    Ok(())
}

fn ensure_database_file(database_url: &str) -> anyhow::Result<()> {
    let Some(path) = sqlite_file_path(database_url) else {
        return Ok(());
    };
    if let Some(parent) = Path::new(&path).parent().filter(|parent| !parent.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent).with_context(|| {
            format!("failed to create database directory {}", parent.display())
        })?;
    }
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .with_context(|| format!("failed to create writable database file: {path}"))?;
    Ok(())
}

fn sqlite_file_path(database_url: &str) -> Option<String> {
    let without_scheme = database_url
        .strip_prefix("sqlite://")
        .or_else(|| database_url.strip_prefix("sqlite:"))?;
    let path = without_scheme.split('?').next().unwrap_or_default();
    if path.is_empty() || path == ":memory:" || path.starts_with("file:") {
        return None;
    }
    Some(path.to_owned())
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
