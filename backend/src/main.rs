use std::net::SocketAddr;
use std::sync::Arc;

use aicte_apaar_api::config::Config;
use aicte_apaar_api::routes;
use aicte_apaar_api::state::AppState;
use sqlx::postgres::PgPoolOptions;
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::trace::TraceLayer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();

    let config = Config::from_env()?;
    tracing::info!(
        env = %config.app_env,
        rule_version = %config.demo_rule_version,
        "starting aicte-apaar-api"
    );

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .acquire_timeout(std::time::Duration::from_secs(10))
        .connect(&config.database_url)
        .await?;

    // Migrations are applied on boot so the schema is always reproducible.
    sqlx::migrate!("./migrations").run(&pool).await?;
    tracing::info!("migrations applied");

    let state = AppState {
        pool,
        config: Arc::new(config),
    };

    let app = routes::router(state)
        .layer(RequestBodyLimitLayer::new(6 * 1024 * 1024))
        .layer(TraceLayer::new_for_http());

    let addr: SocketAddr = app
        .state::<AppState>()
        .map(|s| s.config.bind_addr.clone())
        .unwrap_or_else(|| "127.0.0.1:8080".to_string())
        .parse()?;

    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!(%addr, "listening");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

fn init_tracing() {
    use tracing_subscriber::{fmt, prelude::*, EnvFilter};
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info,sqlx=warn,tower_http=info"));
    tracing_subscriber::registry()
        .with(fmt::layer().with_target(false))
        .with(filter)
        .init();
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("shutdown signal received");
}
