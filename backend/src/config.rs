use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_refresh_secret: String,
    pub port: u16,
}

impl Config {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();
        Self {
            database_url: env::var("DATABASE_URL")
                .expect("DATABASE_URL must be set"),
            jwt_secret: env::var("JWT_SECRET")
                .unwrap_or_else(|_| "dev_jwt_secret_change_in_prod".to_string()),
            jwt_refresh_secret: env::var("JWT_REFRESH_SECRET")
                .unwrap_or_else(|_| "dev_refresh_secret_change_in_prod".to_string()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .expect("PORT must be a number"),
        }
    }
}
