use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Nije pronađeno")]
    NotFound,

    #[error("Slot nije dostupan")]
    SlotUnavailable,

    #[error("Nemate dozvolu za ovu akciju")]
    Forbidden,

    #[error("Nevalidan token")]
    InvalidToken,

    #[error("Greška baze podataka")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Interna greška servera")]
    InternalError,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::NotFound => (StatusCode::NOT_FOUND, self.to_string()),
            AppError::SlotUnavailable => (StatusCode::CONFLICT, self.to_string()),
            AppError::Forbidden => (StatusCode::FORBIDDEN, self.to_string()),
            AppError::InvalidToken => (StatusCode::UNAUTHORIZED, self.to_string()),
            AppError::DatabaseError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Greška baze podataka".to_string()),
            AppError::InternalError => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };
        (status, Json(json!({ "error": message }))).into_response()
    }
}