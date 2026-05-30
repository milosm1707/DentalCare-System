use axum::{
    routing::{get, post},
    Router,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use mongodb::Client;
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
    pub db: mongodb::Database,
    pub config: Config,
}

async fn auth_middleware(
    axum::extract::State(state): axum::extract::State<AppState>,
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

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Config::from_env();

    let client = Client::with_uri_str(&config.mongodb_url)
        .await
        .expect("Nije moguće konekovati se na MongoDB");

    let db = client.database("chat_db");
    tracing::info!("Konekcija na MongoDB uspostavljena");

    let state = AppState {
        db,
        config: config.clone(),
    };

    let protected = Router::new()
        .route("/messages", post(handlers::send_message))
        .route("/messages/{user_id}", get(handlers::get_chat_history))
        .route("/notifications", get(handlers::get_notifications))
        .route("/ws", get(handlers::ws_handler))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ));

    let app = Router::new()
        .route("/health", get(|| async { "OK" }))
        .route("/notifications/internal", post(handlers::create_notification))
        .merge(protected)
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    tracing::info!("Chat service pokrenut na {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}