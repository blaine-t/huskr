use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use axum_login::AuthSession;
use sqlx::SqlitePool;

use crate::{auth::backend::MicrosoftBackend, error::AppError, models::UserResponse};

/// Returns the currently authenticated user (tokens redacted), including interests.
pub async fn me(
    auth_session: AuthSession<MicrosoftBackend>,
    State(pool): State<SqlitePool>,
) -> Result<impl IntoResponse, AppError> {
    let user = match auth_session.user {
        Some(u) => u,
        None => return Ok(StatusCode::UNAUTHORIZED.into_response()),
    };

    let interests: Vec<String> = sqlx::query_scalar(
        r#"
        SELECT i.name
        FROM interests i
        JOIN user_interests ui ON ui.interest_id = i.id
        WHERE ui.user_id = ?1
        ORDER BY i.name
        "#,
    )
    .bind(user.id)
    .fetch_all(&pool)
    .await?;

    Ok(Json(UserResponse::from_user(user, interests)).into_response())
}
