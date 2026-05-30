use std::env;

#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub port: u16,
    pub auth_service_url: String,
}

impl Config {
    pub fn from_env() -> Self {
        dotenv::dotenv().ok();
        Self {
            database_url: env::var("APPOINTMENT_DATABASE_URL")
                .expect("APPOINTMENT_DATABASE_URL mora biti postavljen"),
            jwt_secret: env::var("JWT_SECRET")
                .expect("JWT_SECRET mora biti postavljen"),
            port: env::var("APPOINTMENT_SERVICE_PORT")
                .unwrap_or_else(|_| "3002".to_string())
                .parse()
                .expect("APPOINTMENT_SERVICE_PORT mora biti broj"),
            auth_service_url: env::var("AUTH_SERVICE_URL")
                .unwrap_or_else(|_| "http://localhost:3001".to_string()),
        }
    }
}