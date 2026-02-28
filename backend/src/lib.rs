pub mod api;
pub mod auth;
pub mod db;
pub mod error;
pub mod middleware;
pub mod models;

use axum::extract::FromRef;
use sqlx::SqlitePool;

use crate::auth::backend::MicrosoftBackend;

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub backend: MicrosoftBackend,
}

impl FromRef<AppState> for SqlitePool {
    fn from_ref(state: &AppState) -> Self {
        state.pool.clone()
    }
}

impl FromRef<AppState> for MicrosoftBackend {
    fn from_ref(state: &AppState) -> Self {
        state.backend.clone()
    }
}
