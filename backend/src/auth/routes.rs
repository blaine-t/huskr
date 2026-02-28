use axum::{
    extract::{Query, State},
    response::Redirect,
};
use axum_login::AuthSession;
use serde::Deserialize;

use crate::{
    auth::{backend::MicrosoftBackend, Credentials},
    AppState,
};

pub type AuthSessionType = AuthSession<MicrosoftBackend>;

/// Optional query param on `/auth/login` — frontend passes `?next=/some/path`
/// so after OAuth completes the backend redirects back to that path.
#[derive(Debug, Deserialize)]
pub struct LoginParams {
    pub next: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CallbackParams {
    pub code: String,
    pub state: String,
}

/// Redirect the browser to the Microsoft authorization URL.
/// Stores CSRF state, PKCE verifier, and optional `next` path in the session.
pub async fn login(
    State(state): State<AppState>,
    auth_session: AuthSessionType,
    Query(params): Query<LoginParams>,
) -> Result<Redirect, crate::error::AppError> {
    let (url, csrf, pkce_verifier) = state.backend.authorize_url();

    auth_session
        .session
        .insert("csrf_state", csrf.secret())
        .await
        .map_err(|e| crate::error::AppError::OAuth(format!("session insert csrf_state: {e}")))?;
    auth_session
        .session
        .insert("pkce_verifier", pkce_verifier.secret())
        .await
        .map_err(|e| crate::error::AppError::OAuth(format!("session insert pkce_verifier: {e}")))?;

    // Persist the post-auth destination so callback can redirect back to it.
    if let Some(next) = &params.next {
        auth_session
            .session
            .insert("next", next)
            .await
            .map_err(|e| crate::error::AppError::OAuth(format!("session insert next: {e}")))?;
    }

    tracing::debug!(csrf = %csrf.secret(), next = ?params.next, "initiating Microsoft SSO");
    Ok(Redirect::to(url.as_str()))
}

/// Microsoft redirects here after the user authenticates.
/// Validates CSRF, exchanges the authorization code for tokens,
/// upserts the user in the database, and establishes an axum-login session.
/// Finally redirects the browser back to the frontend SPA.
pub async fn callback(
    State(state): State<AppState>,
    mut auth_session: AuthSessionType,
    Query(params): Query<CallbackParams>,
) -> Result<Redirect, crate::error::AppError> {
    // --- CSRF validation ---
    let stored_state: Option<String> = auth_session.session.get("csrf_state").await.ok().flatten();
    tracing::debug!(stored = ?stored_state, received = %params.state, "csrf check");
    if stored_state.as_deref() != Some(&params.state) {
        return Err(crate::error::AppError::OAuth(format!(
            "csrf mismatch: stored={stored_state:?}, received={}",
            params.state
        )));
    }

    let pkce_verifier: String = auth_session
        .session
        .get("pkce_verifier")
        .await
        .ok()
        .flatten()
        .ok_or_else(|| crate::error::AppError::OAuth("missing pkce_verifier".into()))?;

    // Retrieve optional next-path before consuming the auth exchange.
    let next: Option<String> = auth_session.session.get("next").await.ok().flatten();

    let creds = Credentials {
        code: params.code,
        pkce_verifier,
    };

    let user = auth_session
        .authenticate(creds)
        .await
        .map_err(|e| crate::error::AppError::OAuth(e.to_string()))?
        .ok_or_else(|| crate::error::AppError::OAuth("authentication returned no user".into()))?;

    auth_session.login(&user).await.ok();
    tracing::info!(user_id = user.id, email = ?user.email, "user logged in via Microsoft SSO");

    // Redirect browser back to the frontend, honouring ?next= if present.
    let destination = match next {
        Some(path) if path.starts_with('/') => format!("{}{}", state.frontend_url, path),
        _ => state.frontend_url.clone(),
    };
    Ok(Redirect::to(&destination))
}

/// Destroy the axum-login session and redirect back to the frontend.
pub async fn logout(
    State(state): State<AppState>,
    mut auth_session: AuthSessionType,
) -> Redirect {
    auth_session.logout().await.ok();
    tracing::info!("user logged out");
    Redirect::to(&state.frontend_url)
}
