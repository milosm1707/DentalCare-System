use axum::{
    body::Body,
    extract::State,
    http::{HeaderMap, Method, Request, StatusCode, Uri},
    response::Response,
    routing::any,
    Router,
};
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
use config::Config;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub http_client: reqwest::Client,
}

async fn proxy(
    State(state): State<AppState>,
    method: Method,
    uri: Uri,
    headers: HeaderMap,
    body: Body,
) -> Result<Response, StatusCode> {
    let path = uri.path_and_query()
        .map(|p| p.as_str())
        .unwrap_or("/");

    // Odredi ciljni servis na osnovu putanje
    let target_url = if path.starts_with("/auth") {
        let new_path = path.trim_start_matches("/auth");
        format!("{}{}", state.config.auth_service_url,
                if new_path.is_empty() { "/" } else { new_path })
    } else if path.starts_with("/appointments") || path.starts_with("/clinics") || path.starts_with("/slots") {
        format!("{}{}", state.config.appointment_service_url, path)
    } else if path.starts_with("/media") || path.starts_with("/upload") || path.starts_with("/files") || path.starts_with("/pdf") {
        format!("{}{}", state.config.media_service_url, path)
    } else if path.starts_with("/chat") || path.starts_with("/messages") || path.starts_with("/notifications") {
        format!("{}{}", state.config.chat_service_url, path)
    } else {
        return Err(StatusCode::NOT_FOUND);
    };

    tracing::info!("{} {} -> {}", method, path, target_url);

    // Preuzmi body kao bytes
    let body_bytes = axum::body::to_bytes(body, usize::MAX)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Proslijedi zahtjev
    let mut req_builder = state.http_client
        .request(
            reqwest::Method::from_bytes(method.as_str().as_bytes())
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
            &target_url,
        )
        .body(body_bytes);

    // Proslijedi headers (Authorization, Content-Type, itd.)
    for (key, value) in headers.iter() {
        let key_str = key.as_str();
        if key_str != "host" {
            if let Ok(v) = value.to_str() {
                req_builder = req_builder.header(key_str, v);
            }
        }
    }

    let response = req_builder
        .send()
        .await
        .map_err(|e| {
            tracing::error!("Proxy greška: {}", e);
            StatusCode::BAD_GATEWAY
        })?;

    // Preuzmi response i proslijedi nazad
    let status = StatusCode::from_u16(response.status().as_u16())
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut response_headers = HeaderMap::new();
    for (key, value) in response.headers().iter() {
        if let Ok(k) = axum::http::HeaderName::from_bytes(key.as_str().as_bytes()) {
            if let Ok(v) = axum::http::HeaderValue::from_bytes(value.as_bytes()) {
                response_headers.insert(k, v);
            }
        }
    }

    let response_body = response.bytes()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    let mut axum_response = Response::new(Body::from(response_body));
    *axum_response.status_mut() = status;
    *axum_response.headers_mut() = response_headers;

    Ok(axum_response)
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Config::from_env();

    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .expect("Nije moguće kreirati HTTP klijent");

    let state = AppState {
        config: config.clone(),
        http_client,
    };

    let app = Router::new()
        .route("/health", any(|| async { "API Gateway OK" }))
        .route("/{*path}", any(proxy))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    tracing::info!("API Gateway pokrenut na {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}