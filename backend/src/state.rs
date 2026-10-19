use sqlx::PgPool;
use std::sync::Arc;

use crate::config::Config;

/// Shared application state handed to every handler.
#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: Arc<Config>,
}
