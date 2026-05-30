use std::env;

#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub port: u16,
}

impl Config {
    pub fn from_env() -> Self {
        dotenv::dotenv().ok();
        Self {
            database_url: env::var("AUTH_DATABASE_URL")
                .expect("AUTH_DATABASE_URL mora biti postavljen"),
            jwt_secret: env::var("JWT_SECRET")
                .expect("JWT_SECRET mora biti postavljen"),
            port: env::var("AUTH_SERVICE_PORT")
                .unwrap_or_else(|_| "3001".to_string())
                .parse()
                .expect("AUTH_SERVICE_PORT mora biti broj"),
        }
    }
}