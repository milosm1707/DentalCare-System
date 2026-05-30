use std::env;

#[derive(Clone)]
pub struct Config {
    pub port: u16,
    pub auth_service_url: String,
    pub appointment_service_url: String,
    pub media_service_url: String,
    pub chat_service_url: String,
    pub jwt_secret: String,
}

impl Config {
    pub fn from_env() -> Self {
        dotenv::dotenv().ok();
        Self {
            port: env::var("GATEWAY_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .expect("GATEWAY_PORT mora biti broj"),
            auth_service_url: env::var("AUTH_SERVICE_URL")
                .unwrap_or_else(|_| "http://localhost:3001".to_string()),
            appointment_service_url: env::var("APPOINTMENT_SERVICE_URL")
                .unwrap_or_else(|_| "http://localhost:3002".to_string()),
            media_service_url: env::var("MEDIA_SERVICE_URL")
                .unwrap_or_else(|_| "http://localhost:3003".to_string()),
            chat_service_url: env::var("CHAT_SERVICE_URL")
                .unwrap_or_else(|_| "http://localhost:3004".to_string()),
            jwt_secret: env::var("JWT_SECRET")
                .expect("JWT_SECRET mora biti postavljen"),
        }
    }
}