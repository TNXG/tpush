use aes_gcm::aead::{Aead, KeyInit, OsRng};
use aes_gcm::{AeadCore, Aes256Gcm};
use axum::http::StatusCode;
use base64::Engine;
use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};
use sqlx::{Pool, Sqlite};

use crate::error::ApiError;
use crate::models::EncryptedEnvelope;

type HmacSha256 = Hmac<Sha256>;

#[derive(Clone, Debug)]
pub enum ChannelKey {
    Public,
    Private(String),
}

impl ChannelKey {
    #[allow(dead_code)]
    pub fn is_private(&self) -> bool {
        matches!(self, Self::Private(_))
    }
}

pub fn encrypt_message(
    channel: &str,
    channel_key: &ChannelKey,
    plaintext: &str,
) -> EncryptedEnvelope {
    let ChannelKey::Private(secret) = channel_key else {
        return EncryptedEnvelope {
            version: 1,
            channel: channel.to_owned(),
            algorithm: "none".to_owned(),
            encrypted: false,
            data: plaintext.to_owned(),
        };
    };

    let mut hasher = Sha256::new();
    hasher.update(secret.as_bytes());
    let key: [u8; 32] = hasher.finalize().into();
    let cipher = Aes256Gcm::new(&key.into());
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ciphertext = cipher
        .encrypt(
            &nonce,
            aes_gcm::aead::Payload {
                msg: plaintext.as_bytes(),
                aad: channel.as_bytes(),
            },
        )
        .unwrap_or_default();

    let mut payload = nonce.to_vec();
    payload.extend_from_slice(&ciphertext);
    let base64_data = base64::engine::general_purpose::STANDARD.encode(payload);

    EncryptedEnvelope {
        version: 1,
        channel: channel.to_owned(),
        algorithm: "AES-256-GCM+SHA256".to_owned(),
        encrypted: true,
        data: base64_data,
    }
}

pub async fn load_channel_key(
    database: &Pool<Sqlite>,
    channel: &str,
) -> Result<ChannelKey, ApiError> {
    let channel_name = normalize_channel_name(channel)?;
    let expected_key = sqlx::query_scalar::<_, String>("SELECT key FROM channels WHERE name = ?1")
        .bind(&channel_name)
        .fetch_optional(database)
        .await?;

    let Some(expected_key) = expected_key else {
        tracing::warn!(
            channel = %channel_name,
            reason = "channel_not_found",
            key_check_status = "failed",
            "channel access denied"
        );
        return Err(ApiError::with_status(
            StatusCode::NOT_FOUND,
            "channel not found",
        ));
    };

    let trimmed_key = expected_key.trim().to_owned();
    if trimmed_key.is_empty() {
        tracing::info!(
            channel = %channel_name,
            key_check_status = "public",
            "channel has no key; encryption and auth are disabled"
        );
        return Ok(ChannelKey::Public);
    }

    Ok(ChannelKey::Private(trimmed_key))
}

pub async fn validate_channel_signature(
    database: &Pool<Sqlite>,
    channel: &str,
    subject: &str,
    ts: &str,
    nonce: &str,
    signature: &str,
) -> Result<ChannelKey, ApiError> {
    let channel_key = load_channel_key(database, channel).await?;
    let ChannelKey::Private(secret) = &channel_key else {
        return Ok(channel_key);
    };

    if ts.is_empty() || nonce.is_empty() || signature.is_empty() {
        tracing::warn!(
            channel = %channel,
            subject = %subject,
            reason = "signature_missing",
            key_check_status = "failed",
            "channel access denied: signature fields are missing"
        );
        return Err(ApiError::with_status(
            StatusCode::UNAUTHORIZED,
            "channel signature is missing",
        ));
    }

    let signed_payload = signed_payload(channel, subject, ts, nonce);
    let mut mac = <HmacSha256 as digest::KeyInit>::new_from_slice(secret.as_bytes())
        .map_err(|error| ApiError::from(anyhow::anyhow!(error.to_string())))?;
    mac.update(signed_payload.as_bytes());
    let Ok(signature_bytes) = hex::decode(signature) else {
        tracing::warn!(
            channel = %channel,
            subject = %subject,
            reason = "signature_not_hex",
            key_check_status = "failed",
            "channel access denied: signature is not valid hex"
        );
        return Err(ApiError::with_status(
            StatusCode::UNAUTHORIZED,
            "channel signature format is invalid",
        ));
    };

    if mac.verify_slice(&signature_bytes).is_ok() {
        tracing::info!(
            channel = %channel,
            subject = %subject,
            key_check_status = "ok",
            "channel signature verified"
        );
        return Ok(channel_key);
    }

    tracing::warn!(
        channel = %channel,
        subject = %subject,
        reason = "signature_mismatch",
        key_check_status = "failed",
        "channel access denied: signature mismatch"
    );
    Err(ApiError::with_status(
        StatusCode::UNAUTHORIZED,
        "invalid channel signature",
    ))
}

pub fn normalize_channel_name(channel: &str) -> Result<String, ApiError> {
    let name = channel.trim();
    if name.is_empty() {
        return Err(ApiError::with_status(
            StatusCode::BAD_REQUEST,
            "channel is required",
        ));
    }
    if name.len() > 80
        || !name
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
    {
        return Err(ApiError::with_status(
            StatusCode::BAD_REQUEST,
            "channel may only contain letters, numbers, hyphen and underscore",
        ));
    }
    Ok(name.to_owned())
}

fn signed_payload(channel: &str, subject: &str, ts: &str, nonce: &str) -> String {
    format!("{channel}:{subject}:{ts}:{nonce}")
}
