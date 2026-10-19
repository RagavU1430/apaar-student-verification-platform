use std::env;

/// Runtime configuration, loaded from the environment (`.env` is read at boot).
///
/// Nothing here is secret-by-default: the prototype refuses to start with the
/// example JWT secret when `APP_ENV=production`.
#[derive(Clone, Debug)]
pub struct Config {
    pub bind_addr: String,
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_ttl_minutes: i64,
    pub max_upload_bytes: usize,
    pub max_batch_rows: usize,
    pub default_page_size: i64,
    pub max_page_size: i64,
    pub demo_rule_version: String,
    /// Demo-only shape check for APAAR ids. Level 2 validation only.
    /// Never presented as identity verification. See BUILD_PLAN.md A-01.
    pub apaar_demo_pattern: String,
    pub app_env: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let _ = dotenvy::dotenv();

        let app_env = env_or("APP_ENV", "development");
        let jwt_secret = env_or(
            "JWT_SECRET",
            "dev-only-change-me-0123456789abcdefghijklmnopqrstuvwxyz",
        );
        if app_env.eq_ignore_ascii_case("production") && jwt_secret.starts_with("dev-only") {
            anyhow::bail!("JWT_SECRET must be set to a real secret when APP_ENV=production");
        }

        let database_url = env::var("DATABASE_URL")
            .map_err(|_| anyhow::anyhow!("DATABASE_URL must be set (see .env.example)"))?;

        Ok(Self {
            bind_addr: env_or("BIND_ADDR", "127.0.0.1:8080"),
            database_url,
            jwt_secret,
            jwt_ttl_minutes: parse_or("JWT_TTL_MINUTES", 60),
            max_upload_bytes: parse_or("MAX_UPLOAD_BYTES", 5 * 1024 * 1024),
            max_batch_rows: parse_or("MAX_BATCH_ROWS", 5_000),
            default_page_size: parse_or("DEFAULT_PAGE_SIZE", 50),
            max_page_size: parse_or("MAX_PAGE_SIZE", 200),
            demo_rule_version: env_or("DEMO_RULE_VERSION", "demo-v1"),
            apaar_demo_pattern: env_or("APAAR_DEMO_PATTERN", "^[A-Z0-9-]{8,20}$"),
            app_env,
        })
    }

    pub fn is_production(&self) -> bool {
        self.app_env.eq_ignore_ascii_case("production")
    }
}

fn env_or(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}

fn parse_or<T: std::str::FromStr>(key: &str, default: T) -> T {
    env::var(key)
        .ok()
        .and_then(|v| v.parse::<T>().ok())
        .unwrap_or(default)
}
