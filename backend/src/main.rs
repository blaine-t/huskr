use axum::{
    http::{header, HeaderValue, Method},
    middleware,
    routing::{get, post},
    Router,
};
use axum_login::AuthManagerLayerBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tower_sessions::{MemoryStore, SessionManagerLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use backend::{
    api::{likes::submit_like, user::me},
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
    // Base URL of the Leptos SPA, used for CORS and post-auth redirects.
    let frontend_url =
        std::env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:3001".into());

    let backend =
        MicrosoftBackend::new(pool.clone(), client_id, client_secret, &tenant, redirect_url)?;

    // Session layer — swap MemoryStore for a persistent store in production
    // (e.g. tower-sessions-sqlx-store).
    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store)
        .with_same_site(tower_sessions::cookie::SameSite::Lax);

    let auth_layer = AuthManagerLayerBuilder::new(backend.clone(), session_layer).build();

    let state = AppState {
        pool,
        backend,
        frontend_url: frontend_url.clone(),
    };

    // CORS — must allow credentials so the browser sends the session cookie
    // on cross-origin requests from the Leptos frontend.
    let frontend_origin: HeaderValue = frontend_url
        .parse()
        .expect("FRONTEND_URL is not a valid HTTP origin");
    let cors = CorsLayer::new()
        .allow_origin(frontend_origin)
        .allow_methods([Method::GET, Method::POST, Method::DELETE, Method::OPTIONS])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION, header::ACCEPT])
        .allow_credentials(true);

    let protected = Router::new()
        .route("/api/user/me", get(me))
        .route("/api/likes", post(submit_like))
        .layer(middleware::from_fn_with_state(state.clone(), require_user));

    let app = Router::new()
        .route("/auth/login", get(login))
        .route("/auth/callback", get(callback))
        .route("/auth/logout", get(logout))
        .merge(protected)
        .layer(auth_layer)
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:48757").await?;
    tracing::info!("listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;

    Ok(())
}
