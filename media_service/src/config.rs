use std::env;

#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub port: u16,
    pub upload_dir: String,
}

impl Config {
    pub fn from_env() -> Self {
        dotenv::dotenv().ok();
        Self {
            database_url: env::var("MEDIA_DATABASE_URL")
                .expect("MEDIA_DATABASE_URL mora biti postavljen"),
            jwt_secret: env::var("JWT_SECRET")
                .expect("JWT_SECRET mora biti postavljen"),
            port: env::var("MEDIA_SERVICE_PORT")
                .unwrap_or_else(|_| "3003".to_string())
                .parse()
                .expect("MEDIA_SERVICE_PORT mora biti broj"),
            upload_dir: env::var("UPLOAD_DIR")
                .unwrap_or_else(|_| "./uploads".to_string()),
        }
    }
}