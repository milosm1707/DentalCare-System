use axum::{
    routing::{delete, get, post},
    Router,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod errors;
mod handlers;
mod models;

use config::Config;
use models::Claims;

#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::PgPool,
    pub config: Config,
}

async fn auth_middleware(
    State(state): axum::extract::State<AppState>,
    headers: axum::http::HeaderMap,
    mut request: axum::extract::Request,
    next: axum::middleware::Next,
) -> Result<axum::response::Response, errors::AppError> {
    let token = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .ok_or(errors::AppError::InvalidToken)?;

    let claims = decode::<Claims>(
        token,
        &DecodingKey::from_secret(state.config.jwt_secret.as_bytes()),
        &Validation::default(),
    )
        .map_err(|_| errors::AppError::InvalidToken)?
        .claims;

    request.extensions_mut().insert(claims);
    Ok(next.run(request).await)
}

use axum::extract::State;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Config::from_env();

    let db = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await
        .expect("Nije moguće konekovati se na bazu");

    tracing::info!("Konekcija na bazu uspostavljena");

    let state = AppState {
        db,
        config: config.clone(),
    };

    let protected = Router::new()
        .route("/clinics", post(handlers::create_clinic))
        .route("/slots", post(handlers::create_slot))
        .route("/slots/:dentist_id", get(handlers::get_available_slots))
        .route("/appointments", post(handlers::book_appointment))
        .route("/appointments/my", get(handlers::get_my_appointments))
        .route("/appointments/:id/cancel", delete(handlers::cancel_appointment))
        .route("/reviews", post(handlers::create_review))
        .route("/reviews/my", get(handlers::get_my_reviews))
        .route("/reviews/admin", get(handlers::get_all_reviews_admin))
        .route("/clinics/my", get(handlers::get_my_clinics))
        .route("/appointments/:id/confirm", post(handlers::confirm_appointment))
        .route("/reviews/:id/approve", post(handlers::approve_review))
        .route("/reviews/:id/reject", post(handlers::reject_review))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    let app = Router::new()
        .route("/health", get(|| async { "OK" }))
        .route("/clinics", get(handlers::get_clinics))
        .route("/reviews/dentist/:dentist_id", get(handlers::get_dentist_reviews))
        .merge(protected)
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    tracing::info!("Appointment service pokrenut na {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}