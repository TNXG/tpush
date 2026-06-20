use axum::Json;
use axum::http::StatusCode;

pub struct ApiError {
    pub status: StatusCode,
    pub error: anyhow::Error,
}

impl ApiError {
    pub fn with_status(status: StatusCode, message: impl Into<String>) -> Self {
        Self {
            status,
            error: anyhow::anyhow!(message.into()),
        }
    }
}

impl<E> From<E> for ApiError
where
    E: Into<anyhow::Error>,
{
    fn from(error: E) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            error: error.into(),
        }
    }
}

impl axum::response::IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        tracing::error!("{:?}", self.error);
        (
            self.status,
            Json(serde_json::json!({ "error": self.error.to_string() })),
        )
            .into_response()
    }
}
