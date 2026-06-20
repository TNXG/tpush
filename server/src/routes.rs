use axum::extract::ws::{Message as WebSocketMessage, WebSocket, WebSocketUpgrade};
use axum::extract::{Path, Query, State};
use axum::Json;
use chrono::Utc;
use futures_util::{SinkExt, StreamExt};
use std::collections::HashMap;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::crypto::{encrypt_message, load_channel_key, validate_channel_signature};
use crate::error::ApiError;
use crate::models::{
    DeleteMessagesRequest, DeleteMessagesResponse, EncryptedEnvelope, MessageHistoryItem,
    PushRequest, PushResponse, RealtimeMessage, RegisterDeviceRequest, RegisterDeviceResponse,
    SyncQuery,
};
use crate::state::{AppState, ClientMap};

pub async fn register_device(
    State(state): State<AppState>,
    Json(request): Json<RegisterDeviceRequest>,
) -> Result<Json<RegisterDeviceResponse>, ApiError> {
    validate_channel_signature(
        &state.database,
        &request.channel,
        &request.device_id,
        &request.auth.ts,
        &request.auth.nonce,
        &request.auth.signature,
    )
    .await?;
    let id = Uuid::new_v4().to_string();
    let now = Utc::now();

    sqlx::query(
        r#"
        INSERT INTO devices (id, device_id, channel, created_at, updated_at)
        VALUES (?1, ?2, ?3, ?4, ?5)
        ON CONFLICT(device_id) DO UPDATE SET channel = excluded.channel, updated_at = excluded.updated_at
        "#,
    )
    .bind(&id)
    .bind(&request.device_id)
    .bind(&request.channel)
    .bind(now)
    .bind(now)
    .execute(&state.database)
    .await?;

    Ok(Json(RegisterDeviceResponse { id }))
}

pub async fn stream_channel(
    State(state): State<AppState>,
    Path(channel): Path<String>,
    Query(query): Query<HashMap<String, String>>,
    web_socket_upgrade: WebSocketUpgrade,
) -> Result<axum::response::Response, ApiError> {
    let ts = query.get("ts").cloned().unwrap_or_default();
    let nonce = query.get("nonce").cloned().unwrap_or_default();
    let signature = query.get("signature").cloned().unwrap_or_default();
    validate_channel_signature(&state.database, &channel, "ws", &ts, &nonce, &signature).await?;
    let connection_id = Uuid::new_v4().to_string();
    Ok(web_socket_upgrade
        .on_upgrade(move |socket| handle_channel_socket(state, channel, connection_id, socket)))
}

async fn handle_channel_socket(
    state: AppState,
    channel: String,
    connection_id: String,
    socket: WebSocket,
) {
    let (mut sender, mut receiver) = socket.split();
    let (message_sender, mut message_receiver) = mpsc::unbounded_channel::<String>();
    state
        .clients
        .lock()
        .unwrap()
        .entry(channel.clone())
        .or_default()
        .insert(connection_id.clone(), message_sender);

    loop {
        tokio::select! {
            Some(message) = message_receiver.recv() => {
                if sender.send(WebSocketMessage::Text(message)).await.is_err() {
                    break;
                }
            }
            incoming = receiver.next() => {
                match incoming {
                    Some(Ok(WebSocketMessage::Close(_))) | None => break,
                    Some(Ok(_)) => {}
                    Some(Err(error)) => {
                        tracing::warn!(%channel, %connection_id, ?error, "websocket receive failed");
                        break;
                    }
                }
            }
        }
    }

    if let Some(channel_clients) = state.clients.lock().unwrap().get_mut(&channel) {
        channel_clients.remove(&connection_id);
    }
}

pub async fn push_message(
    State(state): State<AppState>,
    Json(request): Json<PushRequest>,
) -> Result<Json<PushResponse>, ApiError> {
    let channel_key = load_channel_key(&state.database, &request.channel).await?;
    let id = Uuid::new_v4().to_string();
    let created_at = Utc::now();
    let realtime_message = RealtimeMessage {
        id: id.clone(),
        channel: request.channel.clone(),
        title: request.title.clone(),
        content: request.content.clone(),
        payload: request.extras.clone(),
        kind: "server_push".to_owned(),
        created_at: created_at.to_rfc3339(),
    };
    let realtime_message_json = serde_json::to_string(&realtime_message)?;
    let envelope = encrypt_message(&request.channel, &channel_key, &realtime_message_json);
    let envelope_json = serde_json::to_string(&envelope)?;
    let online_deliveries = broadcast_message(&state.clients, &request.channel, envelope_json);
    let delivery_status = if online_deliveries == 0 {
        "queued".to_owned()
    } else {
        format!("online_sent:{online_deliveries}")
    };

    sqlx::query(
        r#"
        INSERT INTO push_messages (id, channel, title, content, extras, delivery_status, created_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
        "#,
    )
    .bind(&id)
    .bind(&request.channel)
    .bind(&request.title)
    .bind(&request.content)
    .bind(request.extras.to_string())
    .bind(delivery_status)
    .bind(created_at)
    .execute(&state.database)
    .await?;

    Ok(Json(PushResponse {
        id,
        accepted: true,
        online_deliveries,
    }))
}

fn broadcast_message(clients: &ClientMap, channel: &str, message_json: String) -> usize {
    let clients = clients.lock().unwrap();
    clients
        .get(channel)
        .into_iter()
        .flat_map(|channel_clients| channel_clients.values())
        .filter(|sender| sender.send(message_json.clone()).is_ok())
        .count()
}

pub async fn list_messages(
    State(state): State<AppState>,
) -> Result<Json<Vec<MessageHistoryItem>>, ApiError> {
    let messages = sqlx::query_as::<_, MessageHistoryItem>(
        r#"
        SELECT id, channel, title, content, extras, delivery_status, created_at
        FROM push_messages
        ORDER BY created_at DESC
        LIMIT 200
        "#,
    )
    .fetch_all(&state.database)
    .await?;
    Ok(Json(messages))
}

pub async fn delete_messages(
    State(state): State<AppState>,
    Json(request): Json<DeleteMessagesRequest>,
) -> Result<Json<DeleteMessagesResponse>, ApiError> {
    let ids = request
        .ids
        .into_iter()
        .map(|id| id.trim().to_owned())
        .filter(|id| !id.is_empty())
        .take(500)
        .collect::<Vec<_>>();

    if ids.is_empty() {
        return Ok(Json(DeleteMessagesResponse { deleted: 0 }));
    }

    let mut query_builder = sqlx::QueryBuilder::new("DELETE FROM push_messages WHERE id IN (");
    let mut separated = query_builder.separated(", ");
    for id in ids {
        separated.push_bind(id);
    }
    separated.push_unseparated(")");

    let result = query_builder.build().execute(&state.database).await?;
    Ok(Json(DeleteMessagesResponse {
        deleted: result.rows_affected(),
    }))
}

pub async fn sync_messages(
    State(state): State<AppState>,
    Query(query): Query<SyncQuery>,
) -> Result<Json<Vec<EncryptedEnvelope>>, ApiError> {
    let channel_key = validate_channel_signature(
        &state.database,
        &query.channel,
        &query.device_id,
        &query.ts,
        &query.nonce,
        &query.signature,
    )
    .await?;
    let messages = query_messages(&state, &query).await?;
    let envelopes = messages
        .into_iter()
        .map(|message| {
            let realtime_message = RealtimeMessage {
                id: message.id,
                channel: message.channel,
                title: message.title,
                content: message.content,
                payload: serde_json::from_str(&message.extras).unwrap_or(serde_json::Value::Null),
                kind: "server_sync".to_owned(),
                created_at: message.created_at.to_rfc3339(),
            };
            let json = serde_json::to_string(&realtime_message).unwrap_or_default();
            encrypt_message(&query.channel, &channel_key, &json)
        })
        .collect();
    Ok(Json(envelopes))
}

async fn query_messages(
    state: &AppState,
    query: &SyncQuery,
) -> Result<Vec<MessageHistoryItem>, ApiError> {
    if let Some(after) = &query.after {
        return Ok(sqlx::query_as::<_, MessageHistoryItem>(
            r#"
            SELECT id, channel, title, content, extras, delivery_status, created_at
            FROM push_messages
            WHERE channel = ?1 AND created_at > ?2
            ORDER BY created_at ASC
            LIMIT 200
            "#,
        )
        .bind(&query.channel)
        .bind(after)
        .fetch_all(&state.database)
        .await?);
    }

    Ok(sqlx::query_as::<_, MessageHistoryItem>(
        r#"
        SELECT id, channel, title, content, extras, delivery_status, created_at
        FROM push_messages
        WHERE channel = ?1
        ORDER BY created_at DESC
        LIMIT 200
        "#,
    )
    .bind(&query.channel)
    .fetch_all(&state.database)
    .await?
    .into_iter()
    .rev()
    .collect())
}
