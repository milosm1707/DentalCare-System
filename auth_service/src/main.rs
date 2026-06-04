use axum::{
    routing::{get, post},
    Router,
};
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use axum::routing::{delete, put};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod errors;
mod auth_middleware;
mod handlers;
mod models;
mod admin_handlers;

use config::Config;

#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::PgPool,
    pub config: Config,
}

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
        .expect("Nije moguće konekovati se na bazu podataka");

    tracing::info!("Konekcija na bazu uspostavljena");

    let state = AppState {
        db,
        config: config.clone(),
    };

    let protected_routes = Router::new()
        .route("/me", get(handlers::get_me))
        .route("/users/search", get(handlers::search_users))
        .route("/users/:id", get(handlers::get_user_by_id))
        .route("/change-password", post(handlers::change_password))
        .route("/dentist-profile", put(handlers::update_dentist_profile))
        // Admin rute
        .route("/admin/stats", get(admin_handlers::get_stats))
        .route("/admin/users", get(admin_handlers::get_all_users))
        .route("/admin/users/:id/block", post(admin_handlers::block_user))
        .route("/admin/users/:id/unblock", post(admin_handlers::unblock_user))
        .route("/admin/users/:id", delete(admin_handlers::delete_user))
        .route("/admin/articles", get(admin_handlers::get_all_articles_admin))
        .route("/admin/articles", post(admin_handlers::create_article))
        .route("/admin/articles/:id", delete(admin_handlers::delete_article))
        .route("/admin/articles/:id/toggle", post(admin_handlers::toggle_article))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            auth_middleware::auth_middleware,
        ));

    let app = Router::new()
        .route("/health", get(|| async { "OK" }))
        .route("/register", post(handlers::register))
        .route("/login", post(handlers::login))
        .route("/refresh", post(handlers::refresh_token))
        .route("/users/:id/dentist-profile", get(handlers::get_dentist_profile))
        .route("/articles", get(admin_handlers::get_articles))
        .merge(protected_routes)
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    tracing::info!("Auth service pokrenut na {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}