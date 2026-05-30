use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Korisnik nije pronađen")]
    NotFound,

    #[error("Pogrešna email adresa ili lozinka")]
    InvalidCredentials,

    #[error("Korisnik sa ovom email adresom već postoji")]
    UserAlreadyExists,

    #[error("Nevalidan ili istekao token")]
    InvalidToken,

    #[error("Nemate dozvolu za ovu akciju")]
    Forbidden,

    #[error("Greška baze podataka: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Interna greška servera")]
    InternalError,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::NotFound => (StatusCode::NOT_FOUND, self.to_string()),
            AppError::InvalidCredentials => (StatusCode::UNAUTHORIZED, self.to_string()),
            AppError::UserAlreadyExists => (StatusCode::CONFLICT, self.to_string()),
            AppError::InvalidToken => (StatusCode::UNAUTHORIZED, self.to_string()),
            AppError::Forbidden => (StatusCode::FORBIDDEN, self.to_string()),
            AppError::DatabaseError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Greška baze podataka".to_string()),
            AppError::InternalError => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        (status, Json(json!({ "error": message }))).into_response()
    }
}