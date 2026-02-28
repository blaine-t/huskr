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

#[derive(Debug, Deserialize)]
pub struct CallbackParams {
    pub code: String,
    pub state: String,
}

pub async fn login(
    State(state): State<AppState>,
    auth_session: AuthSessionType,
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

    tracing::debug!(csrf = %csrf.secret(), "stored csrf_state in session");

    Ok(Redirect::to(url.as_str()))
}

pub async fn callback(
    State(_state): State<AppState>,
    mut auth_session: AuthSessionType,
    Query(params): Query<CallbackParams>,
) -> Result<Redirect, crate::error::AppError> {
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

    Ok(Redirect::to("/home"))
}

pub async fn logout(mut auth_session: AuthSessionType) -> Redirect {
    auth_session.logout().await.ok();
    Redirect::to("/")
}
