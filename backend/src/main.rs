use axum::{
    http::{HeaderValue, Method},
    middleware,
    routing::{get, post},
    Router,
};
use axum_login::AuthManagerLayerBuilder;
use reqwest::header::ACCEPT_ENCODING;
use tower_http::{cors::CorsLayer, services::ServeDir, trace::TraceLayer};
use tower_sessions::{MemoryStore, SessionManagerLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use std::sync::Arc;

use object_store::local::LocalFileSystem;
use axum::http::header::{ACCEPT, ACCEPT_CHARSET, ACCESS_CONTROL_ALLOW_CREDENTIALS, CONTENT_ENCODING, CONTENT_TYPE, ORIGIN, REFERER, SET_COOKIE};

use backend::{
    api::{
        likes::submit_like,
        matches::get_matches,
        messages::{get_message_image, get_messages, send_message},
        profiles::{compatible_profiles, get_profile, get_profile_image},
        user::{me, update_profile},
    },
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

    let store_path = std::env::var("OBJECT_STORE_PATH").unwrap_or_else(|_| "./uploads".into());
    std::fs::create_dir_all(&store_path)?;
    let store: Arc<dyn object_store::ObjectStore> =
        Arc::new(LocalFileSystem::new_with_prefix(&store_path)?);

    let client_id = std::env::var("AZURE_CLIENT_ID")?;
    let client_secret = std::env::var("AZURE_CLIENT_SECRET")?;
    let tenant = std::env::var("AZURE_TENANT_ID").unwrap_or_else(|_| "common".into());
    let redirect_url = std::env::var("REDIRECT_URL")?;
    // Base URL of the SPA (same origin since we serve it with ServeDir).
    let frontend_url =
        std::env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:48757".into());

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
        store,
    };

    // CORS — must allow credentials so the browser sends the session cookie
    // on cross-origin requests from the Leptos frontend.
    let frontend_origin: HeaderValue = frontend_url
        .parse()
        .expect("FRONTEND_URL is not a valid HTTP origin");

    let origins = [
        "http://localhost:8080".parse().unwrap(),     
        "http://0.0.0.0:8080".parse().unwrap(),
        "http://127.0.0.1:8080".parse().unwrap(),
        frontend_origin
    ];

    let cors = CorsLayer::new()
        .allow_origin(origins)
        .allow_methods([Method::GET, Method::POST, Method::DELETE, Method::OPTIONS])
        .allow_headers([ACCEPT_ENCODING,CONTENT_ENCODING, REFERER, ORIGIN, ACCEPT, CONTENT_TYPE, ACCEPT_CHARSET, REFERER, ACCESS_CONTROL_ALLOW_CREDENTIALS])
        .expose_headers([SET_COOKIE, CONTENT_ENCODING, ACCEPT_ENCODING])
        .allow_credentials(true);

    let protected = Router::new()
        .route("/user/me", get(me))
        .route("/user/profile", post(update_profile))
        .route("/like", post(submit_like))
        .route("/matches", get(get_matches))
        .route("/message", post(send_message))
        .route("/messages/{user_id}", get(get_messages))
        .route("/messages/{message_id}/image", get(get_message_image))
        // static segment must be declared before the dynamic :id capture
        .route("/profiles/compatible", get(compatible_profiles))
        .route("/profiles/{id}", get(get_profile))
        .route("/profiles/{id}/image", get(get_profile_image))
        .layer(middleware::from_fn_with_state(state.clone(), require_user));

    let app = Router::new()
        .route("/auth/login", get(login))
        .route("/auth/callback", get(callback))
        .route("/auth/logout", get(logout))
        .merge(protected)
        .layer(auth_layer)
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
        .fallback_service(ServeDir::new("../frontend").append_index_html_on_directories(true));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:48757").await?;
    tracing::info!("listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;

    Ok(())
}
