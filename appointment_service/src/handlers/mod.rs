use axum::{
    extract::{Path, State},
    Json,
};
use uuid::Uuid;

use crate::{
    errors::AppError,
    models::{
        Appointment, AppointmentStatus, AvailableSlot, Claims,
        BookAppointmentRequest, Clinic, CreateClinicRequest, CreateSlotRequest,
    },
    AppState,
};
use crate::models::{CreateReviewRequest, Review};

// Kreiranje ambulante (samo stomatolog)
pub async fn create_clinic(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
    Json(req): Json<CreateClinicRequest>,
) -> Result<Json<Clinic>, AppError> {
    if claims.role != "dentist" {
        return Err(AppError::Forbidden);
    }

    let dentist_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::InvalidToken)?;

    let clinic = sqlx::query_as::<_, Clinic>(
        r#"INSERT INTO clinics (dentist_id, name, address, phone, working_hours_start, working_hours_end)
           VALUES ($1, $2, $3, $4, $5, $6) RETURNING *"#,
    )
        .bind(dentist_id)
        .bind(&req.name)
        .bind(&req.address)
        .bind(&req.phone)
        .bind(req.working_hours_start)
        .bind(req.working_hours_end)
        .fetch_one(&state.db)
        .await?;

    Ok(Json(clinic))
}

// Dohvati sve ambulante
pub async fn get_clinics(
    State(state): State<AppState>,
) -> Result<Json<Vec<Clinic>>, AppError> {
    let clinics = sqlx::query_as::<_, Clinic>("SELECT * FROM clinics ORDER BY created_at DESC")
        .fetch_all(&state.db)
        .await?;

    Ok(Json(clinics))
}

// Kreiranje slobodnog slota (samo stomatolog)
pub async fn create_slot(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
    Json(req): Json<CreateSlotRequest>,
) -> Result<Json<AvailableSlot>, AppError> {
    if claims.role != "dentist" {
        return Err(AppError::Forbidden);
    }

    let dentist_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::InvalidToken)?;

    let slot = sqlx::query_as::<_, AvailableSlot>(
        r#"INSERT INTO available_slots (dentist_id, clinic_id, slot_date, start_time, duration)
           VALUES ($1, $2, $3, $4, $5) RETURNING *"#,
    )
        .bind(dentist_id)
        .bind(req.clinic_id)
        .bind(req.slot_date)
        .bind(req.start_time)
        .bind(&req.duration)
        .fetch_one(&state.db)
        .await?;

    Ok(Json(slot))
}

// Dohvati slobodne slotove za stomatologa
pub async fn get_available_slots(
    State(state): State<AppState>,
    Path(clinic_id): Path<Uuid>,
) -> Result<Json<Vec<AvailableSlot>>, AppError> {
    let slots = sqlx::query_as::<_, AvailableSlot>(
        "SELECT * FROM available_slots WHERE clinic_id = $1 AND is_available = true AND slot_date >= CURRENT_DATE ORDER BY slot_date, start_time"
    )
        .bind(clinic_id)
        .fetch_all(&state.db)
        .await?;

    Ok(Json(slots))
}

pub async fn create_review(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
    Json(req): Json<CreateReviewRequest>,
) -> Result<Json<Review>, AppError> {
    if claims.role != "patient" {
        return Err(AppError::Forbidden);
    }

    if req.rating < 1 || req.rating > 5 {
        return Err(AppError::SlotUnavailable);
    }

    let patient_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::InvalidToken)?;

    let review = sqlx::query_as::<_, Review>(
        r#"INSERT INTO reviews (patient_id, dentist_id, appointment_id, rating, comment)
           VALUES ($1, $2, $3, $4, $5) RETURNING *"#,
    )
        .bind(patient_id)
        .bind(req.dentist_id)
        .bind(req.appointment_id)
        .bind(req.rating)
        .bind(&req.comment)
        .fetch_one(&state.db)
        .await?;

    Ok(Json(review))
}

pub async fn get_dentist_reviews(
    State(state): State<AppState>,
    Path(dentist_id): Path<Uuid>,
) -> Result<Json<Vec<Review>>, AppError> {
    let reviews = sqlx::query_as::<_, Review>(
        "SELECT * FROM reviews WHERE dentist_id = $1 AND status = 'approved' ORDER BY created_at DESC"
    )
        .bind(dentist_id)
        .fetch_all(&state.db)
        .await?;

    Ok(Json(reviews))
}

pub async fn get_my_reviews(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
) -> Result<Json<Vec<Review>>, AppError> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::InvalidToken)?;

    let reviews = if claims.role == "patient" {
        sqlx::query_as::<_, Review>(
            "SELECT * FROM reviews WHERE patient_id = $1 ORDER BY created_at DESC"
        )
            .bind(user_id)
            .fetch_all(&state.db)
            .await?
    } else {
        sqlx::query_as::<_, Review>(
            "SELECT * FROM reviews WHERE dentist_id = $1 ORDER BY created_at DESC"
        )
            .bind(user_id)
            .fetch_all(&state.db)
            .await?
    };

    Ok(Json(reviews))
}

pub async fn get_all_reviews_admin(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
) -> Result<Json<Vec<Review>>, AppError> {
    if claims.role != "admin" {
        return Err(AppError::Forbidden);
    }

    let reviews = sqlx::query_as::<_, Review>(
        "SELECT * FROM reviews WHERE status = 'pending' ORDER BY created_at DESC"
    )
        .fetch_all(&state.db)
        .await?;

    Ok(Json(reviews))
}

pub async fn approve_review(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
    Path(review_id): Path<Uuid>,
) -> Result<Json<Review>, AppError> {
    if claims.role != "admin" {
        return Err(AppError::Forbidden);
    }

    let review = sqlx::query_as::<_, Review>(
        "UPDATE reviews SET status = 'approved', updated_at = NOW() WHERE id = $1 RETURNING *"
    )
        .bind(review_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::NotFound)?;

    Ok(Json(review))
}

pub async fn reject_review(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
    Path(review_id): Path<Uuid>,
) -> Result<Json<Review>, AppError> {
    if claims.role != "admin" {
        return Err(AppError::Forbidden);
    }

    let review = sqlx::query_as::<_, Review>(
        "UPDATE reviews SET status = 'rejected', updated_at = NOW() WHERE id = $1 RETURNING *"
    )
        .bind(review_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::NotFound)?;

    Ok(Json(review))
}

// Zakazivanje termina (samo pacijent)
pub async fn book_appointment(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
    Json(req): Json<BookAppointmentRequest>,
) -> Result<Json<Appointment>, AppError> {
    if claims.role != "patient" {
        return Err(AppError::Forbidden);
    }

    let patient_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::InvalidToken)?;

    // Provjeri dostupnost slota
    let slot = sqlx::query_as::<_, AvailableSlot>(
        "SELECT * FROM available_slots WHERE id = $1 AND is_available = true"
    )
        .bind(req.slot_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::SlotUnavailable)?;

    // Kreiraj termin
    let appointment = sqlx::query_as::<_, Appointment>(
        r#"INSERT INTO appointments
           (patient_id, dentist_id, clinic_id, slot_id, notes, appointment_date, start_time, duration)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING *"#,
    )
        .bind(patient_id)
        .bind(slot.dentist_id)
        .bind(slot.clinic_id)
        .bind(slot.id)
        .bind(&req.notes)
        .bind(slot.slot_date)
        .bind(slot.start_time)
        .bind(&slot.duration)
        .fetch_one(&state.db)
        .await?;

    // Označi slot kao zauzet
    sqlx::query("UPDATE available_slots SET is_available = false WHERE id = $1")
        .bind(slot.id)
        .execute(&state.db)
        .await?;

    Ok(Json(appointment))
}

// Dohvati termine pacijenta
pub async fn get_my_appointments(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
) -> Result<Json<Vec<Appointment>>, AppError> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::InvalidToken)?;

    let appointments = if claims.role == "patient" {
        sqlx::query_as::<_, Appointment>(
            "SELECT * FROM appointments WHERE patient_id = $1 ORDER BY appointment_date DESC"
        )
            .bind(user_id)
            .fetch_all(&state.db)
            .await?
    } else {
        sqlx::query_as::<_, Appointment>(
            "SELECT * FROM appointments WHERE dentist_id = $1 ORDER BY appointment_date DESC"
        )
            .bind(user_id)
            .fetch_all(&state.db)
            .await?
    };

    Ok(Json(appointments))
}

// Otkaži termin
pub async fn cancel_appointment(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
    Path(appointment_id): Path<Uuid>,
) -> Result<Json<Appointment>, AppError> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::InvalidToken)?;

    let appointment = sqlx::query_as::<_, Appointment>(
        "SELECT * FROM appointments WHERE id = $1"
    )
        .bind(appointment_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::NotFound)?;

    // Samo vlasnik termina može otkazati
    if appointment.patient_id != user_id && appointment.dentist_id != user_id {
        return Err(AppError::Forbidden);
    }

    let updated = sqlx::query_as::<_, Appointment>(
        "UPDATE appointments SET status = $1, updated_at = NOW() WHERE id = $2 RETURNING *"
    )
        .bind(AppointmentStatus::Cancelled)
        .bind(appointment_id)
        .fetch_one(&state.db)
        .await?;

    // Oslobodi slot
    sqlx::query("UPDATE available_slots SET is_available = true WHERE id = $1")
        .bind(appointment.slot_id)
        .execute(&state.db)
        .await?;

    Ok(Json(updated))
}