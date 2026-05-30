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