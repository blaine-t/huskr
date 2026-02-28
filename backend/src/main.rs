use axum::{middleware, routing::get, Router};
use axum_login::AuthManagerLayerBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tower_sessions::{MemoryStore, SessionManagerLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use backend::{
    api::user::me,
    auth::{
        backend::MicrosoftBackend,
        routes::{callback, login, logout},
    },
    db::init_pool,
    middleware::require_user,
    AppState,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:app.db".into());
    let pool = init_pool(&database_url).await?;

    let client_id = std::env::var("AZURE_CLIENT_ID")?;
    let client_secret = std::env::var("AZURE_CLIENT_SECRET")?;
    let tenant = std::env::var("AZURE_TENANT_ID").unwrap_or_else(|_| "common".into());
    let redirect_url = std::env::var("REDIRECT_URL")?;

    let backend = MicrosoftBackend::new(pool.clone(), client_id, client_secret, &tenant, redirect_url)?;

    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store);
    let auth_layer = AuthManagerLayerBuilder::new(backend.clone(), session_layer).build();

    let state = AppState { pool, backend };

    let protected = Router::new()
        .route("/api/user/me", get(me))
        .layer(middleware::from_fn_with_state(state.clone(), require_user));

    let app = Router::new()
        .route("/auth/login", get(login))
        .route("/auth/callback", get(callback))
        .route("/auth/logout", get(logout))
        .merge(protected)
        .layer(auth_layer)
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    tracing::info!("listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;

    Ok(())
}
