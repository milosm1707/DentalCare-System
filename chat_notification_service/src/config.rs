use std::env;

#[derive(Clone)]
pub struct Config {
    pub mongodb_url: String,
    pub jwt_secret: String,
    pub port: u16,
}

impl Config {
    pub fn from_env() -> Self {
        dotenv::dotenv().ok();
        Self {
            mongodb_url: env::var("MONGODB_URL")
                .expect("MONGODB_URL mora biti postavljen"),
            jwt_secret: env::var("JWT_SECRET")
                .expect("JWT_SECRET mora biti postavljen"),
            port: env::var("CHAT_SERVICE_PORT")
                .unwrap_or_else(|_| "3004".to_string())
                .parse()
                .expect("CHAT_SERVICE_PORT mora biti broj"),
        }
    }
}