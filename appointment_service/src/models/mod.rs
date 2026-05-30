use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Type;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Type, Clone, PartialEq)]
#[sqlx(type_name = "appointment_status", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum AppointmentStatus {
    Scheduled,
    Confirmed,
    Cancelled,
    Completed,
}

#[derive(Debug, Serialize, Deserialize, Type, Clone, PartialEq)]
#[sqlx(type_name = "slot_duration", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum SlotDuration {
    Thirty,
    Sixty,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Clinic {
    pub id: Uuid,
    pub dentist_id: Uuid,
    pub name: String,
    pub address: String,
    pub phone: Option<String>,
    pub working_hours_start: NaiveTime,
    pub working_hours_end: NaiveTime,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct AvailableSlot {
    pub id: Uuid,
    pub dentist_id: Uuid,
    pub clinic_id: Uuid,
    pub slot_date: NaiveDate,
    pub start_time: NaiveTime,
    pub duration: SlotDuration,
    pub is_available: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Appointment {
    pub id: Uuid,
    pub patient_id: Uuid,
    pub dentist_id: Uuid,
    pub clinic_id: Uuid,
    pub slot_id: Uuid,
    pub status: AppointmentStatus,
    pub notes: Option<String>,
    pub appointment_date: NaiveDate,
    pub start_time: NaiveTime,
    pub duration: SlotDuration,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Request modeli
#[derive(Debug, Deserialize)]
pub struct CreateClinicRequest {
    pub name: String,
    pub address: String,
    pub phone: Option<String>,
    pub working_hours_start: NaiveTime,
    pub working_hours_end: NaiveTime,
}

#[derive(Debug, Deserialize)]
pub struct CreateSlotRequest {
    pub clinic_id: Uuid,
    pub slot_date: NaiveDate,
    pub start_time: NaiveTime,
    pub duration: SlotDuration,
}

#[derive(Debug, Deserialize)]
pub struct BookAppointmentRequest {
    pub slot_id: Uuid,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub role: String,
    pub exp: usize,
    pub iat: usize,
}