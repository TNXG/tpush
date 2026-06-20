use axum::Json;
use axum::extract::{Path, State};
use chrono::Utc;
use uuid::Uuid;

use crate::crypto::normalize_channel_name;
use crate::error::ApiError;
use crate::models::{ChannelItem, CreateChannelRequest, DeleteChannelResponse};
use crate::state::AppState;

pub async fn list_channels(
    State(state): State<AppState>,
) -> Result<Json<Vec<ChannelItem>>, ApiError> {
    let channels = sqlx::query_as::<_, ChannelItem>(
        r#"
        SELECT id, name, key, created_at, updated_at
        FROM channels
        ORDER BY name ASC
        "#,
    )
    .fetch_all(&state.database)
    .await?;
    Ok(Json(channels))
}

pub async fn create_channel(
    State(state): State<AppState>,
    Json(request): Json<CreateChannelRequest>,
) -> Result<Json<ChannelItem>, ApiError> {
    let name = normalize_channel_name(&request.name)?;
    let key = request.key.trim();
    let now = Utc::now();
    let id = Uuid::new_v4().to_string();
    sqlx::query(
        r#"
        INSERT INTO channels (id, name, key, created_at, updated_at)
        VALUES (?1, ?2, ?3, ?4, ?5)
        ON CONFLICT(name) DO UPDATE SET key = excluded.key, updated_at = excluded.updated_at
        "#,
    )
    .bind(&id)
    .bind(&name)
    .bind(key)
    .bind(now)
    .bind(now)
    .execute(&state.database)
    .await?;

    let channel = sqlx::query_as::<_, ChannelItem>(
        "SELECT id, name, key, created_at, updated_at FROM channels WHERE name = ?1",
    )
    .bind(name)
    .fetch_one(&state.database)
    .await?;
    Ok(Json(channel))
}

pub async fn delete_channel(
    State(state): State<AppState>,
    Path(channel): Path<String>,
) -> Result<Json<DeleteChannelResponse>, ApiError> {
    let channel_name = normalize_channel_name(&channel)?;
    let mut transaction = state.database.begin().await?;

    let deleted_messages = sqlx::query("DELETE FROM push_messages WHERE channel = ?1")
        .bind(&channel_name)
        .execute(&mut *transaction)
        .await?
        .rows_affected();

    sqlx::query("UPDATE devices SET channel = 'default', updated_at = ?1 WHERE channel = ?2")
        .bind(Utc::now())
        .bind(&channel_name)
        .execute(&mut *transaction)
        .await?;

    let deleted_channel = sqlx::query("DELETE FROM channels WHERE name = ?1")
        .bind(&channel_name)
        .execute(&mut *transaction)
        .await?
        .rows_affected()
        > 0;

    transaction.commit().await?;
    state.clients.lock().unwrap().remove(&channel_name);

    Ok(Json(DeleteChannelResponse {
        deleted_channel,
        deleted_messages,
    }))
}
