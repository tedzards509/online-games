use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Room or replay not found")]
    NotFound,
    #[error("Invalid room token")]
    Unauthorized,
    #[error("{0}")]
    BadRequest(String),
    #[error("Room changed; try again")]
    Conflict,
    #[error("Storage error")]
    Storage(#[from] sqlx::Error),
    #[error("Stored data is invalid")]
    Json(#[from] serde_json::Error),
    #[error("Database migration failed")]
    Migration(#[from] sqlx::migrate::MigrateError),
}

impl From<arena_core::Error> for ApiError {
    fn from(error: arena_core::Error) -> Self {
        Self::BadRequest(error.to_string())
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorBody {
    pub error: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match &self {
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::Conflict => StatusCode::CONFLICT,
            _ => {
                tracing::error!(error = ?self, "request failed");
                StatusCode::INTERNAL_SERVER_ERROR
            }
        };
        (
            status,
            Json(ErrorBody {
                error: self.to_string(),
            }),
        )
            .into_response()
    }
}
