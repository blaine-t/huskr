use axum::{Json, http::StatusCode, response::IntoResponse};
use axum_login::AuthSession;

use crate::auth::backend::MicrosoftBackend;

pub async fn me(auth_session: AuthSession<MicrosoftBackend>) -> impl IntoResponse {
    match auth_session.user {
        Some(user) => Json(user).into_response(),
        None => StatusCode::UNAUTHORIZED.into_response(),
    }
}
