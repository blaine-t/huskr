use axum::{Json, http::StatusCode, response::IntoResponse};
use axum_login::AuthSession;

use crate::{auth::backend::MicrosoftBackend, models::UserResponse};

/// Returns the currently authenticated user (tokens redacted).
/// The frontend calls this endpoint to check session state.
pub async fn me(auth_session: AuthSession<MicrosoftBackend>) -> impl IntoResponse {
    match auth_session.user {
        Some(user) => Json(UserResponse::from(user)).into_response(),
        None => StatusCode::UNAUTHORIZED.into_response(),
    }
}
