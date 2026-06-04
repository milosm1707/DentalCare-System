use axum::{extract::{Path, State}, Json};
use uuid::Uuid;
use crate::{errors::AppError, models::Claims, AppState};

#[derive(serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct EducationArticle {
    pub id: Uuid,
    pub title: String,
    pub icon: String,
    pub category: String,
    pub summary: String,
    pub content: String,
    pub is_published: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(serde::Deserialize)]
pub struct CreateArticleRequest {
    pub title: String,
    pub icon: String,
    pub category: String,
    pub summary: String,
    pub content: String,
    pub is_published: bool,
}

// ===== STATISTIKE =====
pub async fn get_stats(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
) -> Result<Json<serde_json::Value>, AppError> {
    if claims.role != "admin" {
        return Err(AppError::Forbidden);
    }

    let total_users = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users")
        .fetch_one(&state.db).await?;

    let total_patients = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM users WHERE role = 'patient'"
    ).fetch_one(&state.db).await?;

    let total_dentists = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM users WHERE role = 'dentist'"
    ).fetch_one(&state.db).await?;

    let active_users = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM users WHERE is_active = true"
    ).fetch_one(&state.db).await?;

    Ok(Json(serde_json::json!({
        "total_users": total_users,
        "total_patients": total_patients,
        "total_dentists": total_dentists,
        "active_users": active_users,
    })))
}

// ===== UPRAVLJANJE KORISNICIMA =====
pub async fn get_all_users(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
) -> Result<Json<serde_json::Value>, AppError> {
    if claims.role != "admin" {
        return Err(AppError::Forbidden);
    }

    let users = sqlx::query_as::<_, crate::models::User>(
        "SELECT * FROM users ORDER BY created_at DESC"
    )
        .fetch_all(&state.db)
        .await?;

    let result: Vec<serde_json::Value> = users.iter().map(|u| serde_json::json!({
        "id": u.id,
        "email": u.email,
        "first_name": u.first_name,
        "last_name": u.last_name,
        "role": format!("{:?}", u.role).to_lowercase(),
        "is_active": u.is_active,
        "created_at": u.created_at,
    })).collect();

    Ok(Json(serde_json::json!(result)))
}

pub async fn block_user(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    if claims.role != "admin" {
        return Err(AppError::Forbidden);
    }

    sqlx::query("UPDATE users SET is_active = false, updated_at = NOW() WHERE id = $1")
        .bind(user_id)
        .execute(&state.db)
        .await?;

    Ok(Json(serde_json::json!({ "message": "Korisnik blokiran" })))
}

pub async fn unblock_user(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    if claims.role != "admin" {
        return Err(AppError::Forbidden);
    }

    sqlx::query("UPDATE users SET is_active = true, updated_at = NOW() WHERE id = $1")
        .bind(user_id)
        .execute(&state.db)
        .await?;

    Ok(Json(serde_json::json!({ "message": "Korisnik deblokiran" })))
}

pub async fn delete_user(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    if claims.role != "admin" {
        return Err(AppError::Forbidden);
    }

    sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user_id)
        .execute(&state.db)
        .await?;

    Ok(Json(serde_json::json!({ "message": "Korisnik obrisan" })))
}

// ===== EDUKATIVNI SADRŽAJ =====
pub async fn get_articles(
    State(state): State<AppState>,
) -> Result<Json<Vec<EducationArticle>>, AppError> {
    let articles = sqlx::query_as::<_, EducationArticle>(
        "SELECT * FROM education_articles WHERE is_published = true ORDER BY created_at DESC"
    )
        .fetch_all(&state.db)
        .await?;

    Ok(Json(articles))
}

pub async fn get_all_articles_admin(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
) -> Result<Json<Vec<EducationArticle>>, AppError> {
    if claims.role != "admin" {
        return Err(AppError::Forbidden);
    }

    let articles = sqlx::query_as::<_, EducationArticle>(
        "SELECT * FROM education_articles ORDER BY created_at DESC"
    )
        .fetch_all(&state.db)
        .await?;

    Ok(Json(articles))
}

pub async fn create_article(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
    Json(req): Json<CreateArticleRequest>,
) -> Result<Json<EducationArticle>, AppError> {
    if claims.role != "admin" {
        return Err(AppError::Forbidden);
    }

    let admin_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::InvalidToken)?;

    let article = sqlx::query_as::<_, EducationArticle>(
        r#"INSERT INTO education_articles (title, icon, category, summary, content, is_published, created_by)
           VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING *"#
    )
        .bind(&req.title)
        .bind(&req.icon)
        .bind(&req.category)
        .bind(&req.summary)
        .bind(&req.content)
        .bind(req.is_published)
        .bind(admin_id)
        .fetch_one(&state.db)
        .await?;

    Ok(Json(article))
}

pub async fn delete_article(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
    Path(article_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    if claims.role != "admin" {
        return Err(AppError::Forbidden);
    }

    sqlx::query("DELETE FROM education_articles WHERE id = $1")
        .bind(article_id)
        .execute(&state.db)
        .await?;

    Ok(Json(serde_json::json!({ "message": "Članak obrisan" })))
}

pub async fn toggle_article(
    State(state): State<AppState>,
    axum::Extension(claims): axum::Extension<Claims>,
    Path(article_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, AppError> {
    if claims.role != "admin" {
        return Err(AppError::Forbidden);
    }

    sqlx::query(
        "UPDATE education_articles SET is_published = NOT is_published, updated_at = NOW() WHERE id = $1"
    )
        .bind(article_id)
        .execute(&state.db)
        .await?;

    Ok(Json(serde_json::json!({ "message": "Status ažuriran" })))
}