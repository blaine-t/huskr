pub mod api;
pub mod auth;
pub mod db;
pub mod error;
pub mod middleware;
pub mod models;

use std::sync::Arc;

use axum::extract::FromRef;
use object_store::ObjectStore;
use sqlx::SqlitePool;

use crate::auth::backend::MicrosoftBackend;

/// Shared application state threaded through Axum handlers.
#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub backend: MicrosoftBackend,
    /// Base URL of the Leptos frontend (e.g. `http://localhost:3001`).
    /// Used for post-auth redirects and CORS allow-origin.
    pub frontend_url: String,
    pub store: Arc<dyn ObjectStore>,
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
