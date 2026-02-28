use axum::{extract::Request, middleware::Next, response::Response, http::StatusCode};
use axum_login::AuthSession;

use crate::auth::backend::MicrosoftBackend;

pub async fn require_user(
    auth_session: AuthSession<MicrosoftBackend>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    if auth_session.user.is_none() {
        return Err(StatusCode::UNAUTHORIZED);
    }
    Ok(next.run(request).await)
}
