use axum::{extract::State, Json};
use bcrypt::{hash, verify, DEFAULT_COST};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use uuid::Uuid;

use crate::{
    errors::AppError,
    models::{
        AuthResponse, Claims, LoginRequest, RefreshRequest,
        RegisterRequest, User, UserInfo, UserRole,
    },
    AppState,
};
use crate::models::{ChangePasswordRequest, DentistProfileRow, UpdateProfileRequest};

pub async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    // Provjeri da li korisnik već postoji
    let existing = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM users WHERE email = $1"
    )
        .bind(&req.email)
        .fetch_one(&state.db)
        .await?;

    if existing > 0 {
        return Err(AppError::UserAlreadyExists);
    }

    // Hash lozinke
    let password_hash = hash(&req.password, DEFAULT_COST)
        .map_err(|_| AppError::InternalError)?;

    // Kreiraj korisnika
    let user = sqlx::query_as::<_, User>(
        r#"INSERT INTO users
           (email, password_hash, role, first_name, last_name, phone, date_of_birth)
           VALUES ($1, $2, $3, $4, $5, $6, $7)
           RETURNING *"#,
    )
        .bind(&req.email)
        .bind(&password_hash)
        .bind(&req.role)
        .bind(&req.first_name)
        .bind(&req.last_name)
        .bind(&req.phone)
        .bind(&req.date_of_birth)
        .fetch_one(&state.db)
        .await?;

    // Ako je stomatolog, kreiraj profil
    if user.role == UserRole::Dentist {
        sqlx::query("INSERT INTO dentist_profiles (user_id) VALUES ($1)")
            .bind(user.id)
            .execute(&state.db)
            .await?;
    }

    let (access_token, refresh_token) = generate_tokens(&user, &state.config.jwt_secret)?;
    save_refresh_token(&state.db, user.id, &refresh_token).await?;

    Ok(Json(AuthResponse {
        access_token,
        refresh_token,
        user: UserInfo {
            id: user.id,
            email: user.email,
            role: user.role,
            first_name: user.first_name,
            last_name: user.last_name,
        },
    }))
}

pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, AppError> {
    let user = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE email = $1 AND is_active = true"
    )
        .bind(&req.email)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::InvalidCredentials)?;

    let valid = verify(&req.password, &user.password_hash)
        .map_err(|_| AppError::InternalError)?;

    if !valid {
        return Err(AppError::InvalidCredentials);
    }

    let (access_token, refresh_token) = generate_tokens(&user, &state.config.jwt_secret)?;
    save_refresh_token(&state.db, user.id, &refresh_token).await?;

    Ok(Json(AuthResponse {
        access_token,
        refresh_token,
        user: UserInfo {
            id: user.id,
            email: user.email,
            role: user.role,
            first_name: user.first_name,
            last_name: user.last_name,
        },
    }))
}

pub async fn refresh_token(
    State(state): State<AppState>,
    Json(req): Json<RefreshRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let row = sqlx::query_as::<_, (uuid::Uuid,)>(
        "SELECT user_id FROM refresh_tokens WHERE token = $1 AND expires_at > NOW()"
    )
        .bind(&req.refresh_token)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::InvalidToken)?;

    let user = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE id = $1 AND is_active = true"
    )
        .bind(row.0)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::NotFound)?;

    let (access_token, new_refresh_token) = generate_tokens(&user, &state.config.jwt_secret)?;

    // Obriši stari, sačuvaj novi refresh token
    sqlx::query("DELETE FROM refresh_tokens WHERE token = $1")
        .bind(&req.refresh_token)
        .execute(&state.db)
        .await?;
    save_refresh_token(&state.db, user.id, &new_refresh_token).await?;

    Ok(Json(serde_json::json!({
        "access_token": access_token,
        "refresh_token": new_refresh_token
    })))
}

pub async fn get_me(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
) -> Result<Json<User>, AppError> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::InvalidToken)?;

    let user = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE id = $1"
    )
        .bind(user_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::NotFound)?;

    Ok(Json(user))
}

fn generate_tokens(user: &User, secret: &str) -> Result<(String, String), AppError> {
    let now = Utc::now();

    let access_claims = Claims {
        sub: user.id.to_string(),
        role: format!("{:?}", user.role).to_lowercase(),
        exp: (now + Duration::hours(1)).timestamp() as usize,
        iat: now.timestamp() as usize,
    };

    let access_token = encode(
        &Header::default(),
        &access_claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
        .map_err(|_| AppError::InternalError)?;

    let refresh_token = Uuid::new_v4().to_string();

    Ok((access_token, refresh_token))
}

async fn save_refresh_token(
    db: &sqlx::PgPool,
    user_id: Uuid,
    token: &str,
) -> Result<(), AppError> {
    let expires_at = Utc::now() + Duration::days(30);
    sqlx::query(
        "INSERT INTO refresh_tokens (user_id, token, expires_at) VALUES ($1, $2, $3)"
    )
        .bind(user_id)
        .bind(token)
        .bind(expires_at)
        .execute(db)
        .await?;
    Ok(())
}
pub async fn get_user_by_id(
    State(state): State<AppState>,
    axum::Extension(_claims): axum::Extension<Claims>,
    axum::extract::Path(user_id): axum::extract::Path<uuid::Uuid>,
) -> Result<Json<UserInfo>, AppError> {
    let user = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE id = $1 AND is_active = true"
    )
        .bind(user_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::NotFound)?;

    Ok(Json(UserInfo {
        id: user.id,
        email: user.email,
        role: user.role,
        first_name: user.first_name,
        last_name: user.last_name,
    }))
}
pub async fn search_users(
    State(state): State<AppState>,
    axum::Extension(_claims): axum::Extension<Claims>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> Result<Json<Vec<UserInfo>>, AppError> {
    let query = params.get("q").cloned().unwrap_or_default();

    let users = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE (email ILIKE $1 OR first_name ILIKE $1 OR last_name ILIKE $1) AND is_active = true LIMIT 10"
    )
        .bind(format!("%{}%", query))
        .fetch_all(&state.db)
        .await?;

    Ok(Json(users.into_iter().map(|u| UserInfo {
        id: u.id,
        email: u.email,
        role: u.role,
        first_name: u.first_name,
        last_name: u.last_name,
    }).collect()))
}
pub async fn change_password(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
    Json(req): Json<ChangePasswordRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::InvalidToken)?;

    let user = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE id = $1"
    )
        .bind(user_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::NotFound)?;

    let valid = verify(&req.current_password, &user.password_hash)
        .map_err(|_| AppError::InternalError)?;

    if !valid {
        return Err(AppError::InvalidCredentials);
    }

    let new_hash = hash(&req.new_password, DEFAULT_COST)
        .map_err(|_| AppError::InternalError)?;

    sqlx::query(
        "UPDATE users SET password_hash = $1, updated_at = NOW() WHERE id = $2"
    )
        .bind(&new_hash)
        .bind(user_id)
        .execute(&state.db)
        .await?;

    Ok(Json(serde_json::json!({ "message": "Lozinka uspješno promijenjena" })))
}

pub async fn get_dentist_profile(
    State(state): State<AppState>,
    axum::extract::Path(user_id): axum::extract::Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    let user = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE id = $1 AND role = 'dentist' AND is_active = true"
    )
        .bind(user_id)
        .fetch_optional(&state.db)
        .await?
        .ok_or(AppError::NotFound)?;

    let row = sqlx::query_as::<_, DentistProfileRow>(
        "SELECT specialization, bio, clinic_name, clinic_address, working_hours_start::text, working_hours_end::text FROM dentist_profiles WHERE user_id = $1"
    )
        .bind(user_id)
        .fetch_optional(&state.db)
        .await?;

    let result = serde_json::json!({
        "id": user.id,
        "email": user.email,
        "first_name": user.first_name,
        "last_name": user.last_name,
        "phone": user.phone,
        "specialization": row.as_ref().and_then(|p| p.specialization.as_ref()),
        "bio": row.as_ref().and_then(|p| p.bio.as_ref()),
        "clinic_name": row.as_ref().and_then(|p| p.clinic_name.as_ref()),
        "clinic_address": row.as_ref().and_then(|p| p.clinic_address.as_ref()),
        "working_hours_start": row.as_ref().and_then(|p| p.working_hours_start.as_ref()),
        "working_hours_end": row.as_ref().and_then(|p| p.working_hours_end.as_ref()),
    });

    Ok(Json(result))
}

pub async fn update_dentist_profile(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
    Json(req): Json<UpdateProfileRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    if claims.role != "dentist" {
        return Err(AppError::Forbidden);
    }

    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::InvalidToken)?;

    sqlx::query(
        "UPDATE users SET first_name = $1, last_name = $2, phone = $3, updated_at = NOW() WHERE id = $4"
    )
        .bind(&req.first_name)
        .bind(&req.last_name)
        .bind(&req.phone)
        .bind(user_id)
        .execute(&state.db)
        .await?;

    sqlx::query(
        r#"INSERT INTO dentist_profiles (user_id, specialization, bio, clinic_name, clinic_address, working_hours_start, working_hours_end)
           VALUES ($1, $2, $3, $4, $5, $6::time, $7::time)
           ON CONFLICT (user_id) DO UPDATE SET
               specialization = EXCLUDED.specialization,
               bio = EXCLUDED.bio,
               clinic_name = EXCLUDED.clinic_name,
               clinic_address = EXCLUDED.clinic_address,
               working_hours_start = EXCLUDED.working_hours_start,
               working_hours_end = EXCLUDED.working_hours_end"#
    )
        .bind(user_id)
        .bind(&req.specialization)
        .bind(&req.bio)
        .bind(&req.clinic_name)
        .bind(&req.clinic_address)
        .bind(&req.working_hours_start)
        .bind(&req.working_hours_end)
        .execute(&state.db)
        .await?;

    // Dohvati ažuriranog korisnika i vrati ga
    let updated_user = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE id = $1"
    )
        .bind(user_id)
        .fetch_one(&state.db)
        .await?;

    Ok(Json(serde_json::json!({
        "message": "Profil ažuriran",
        "user": {
            "id": updated_user.id,
            "email": updated_user.email,
            "role": format!("{:?}", updated_user.role).to_lowercase(),
            "first_name": updated_user.first_name,
            "last_name": updated_user.last_name
        }
    })))
}