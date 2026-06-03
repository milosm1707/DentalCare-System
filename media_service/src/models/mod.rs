use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Type;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Type, Clone, PartialEq)]
#[sqlx(type_name = "media_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum MediaType {
    Xray,
    ProfileImage,
    ChatImage,
    PdfConfirmation,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct MediaFile {
    pub id: Uuid,
    pub owner_id: Uuid,
    pub file_name: String,
    pub file_path: String,
    pub media_type: MediaType,
    pub mime_type: String,
    pub file_size: i64,
    pub appointment_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub role: String,
    pub exp: usize,
    pub iat: usize,
}