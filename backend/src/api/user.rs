use axum::{Json, extract::{Multipart, State}, http::StatusCode, response::IntoResponse};
use axum_login::AuthSession;
use object_store::{ObjectStoreExt, PutPayload, path::Path as StorePath};
use sqlx::SqlitePool;

use crate::{AppState, auth::backend::MicrosoftBackend, error::AppError, models::{User, UserResponse}};

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

// ---------------------------------------------------------------------------
// POST /user/profile — update bio and/or profile image
// ---------------------------------------------------------------------------

/// Accepts multipart/form-data with optional fields:
///   - `bio`   — plain text biography
///   - `image` — image file (stored in object_store)
///
/// Only provided fields are updated; omitted fields keep their current value.
pub async fn update_profile(
    auth_session: AuthSession<MicrosoftBackend>,
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    let user = auth_session.user.ok_or(AppError::Unauthorized)?;

    let mut bio: Option<String> = None;
    let mut image_key: Option<String> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
    {
        match field.name().unwrap_or("") {
            "bio" => {
                bio = Some(
                    field
                        .text()
                        .await
                        .map_err(|e| AppError::Internal(e.to_string()))?,
                );
            }
            "image" => {
                let data = field
                    .bytes()
                    .await
                    .map_err(|e| AppError::Internal(e.to_string()))?;

                let key = format!("profiles/{}", user.id);
                let path = StorePath::from(key.as_str());

                state
                    .store
                    .put(&path, PutPayload::from(data))
                    .await
                    .map_err(|e| AppError::Internal(e.to_string()))?;

                image_key = Some(key);
            }
            _ => {}
        }
    }

    sqlx::query(
        r#"
        UPDATE users
        SET bio       = COALESCE(?1, bio),
            image_key = COALESCE(?2, image_key),
            updated_at = datetime('now')
        WHERE id = ?3
        "#,
    )
    .bind(&bio)
    .bind(&image_key)
    .bind(user.id)
    .execute(&state.pool)
    .await?;

    let updated = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?1")
        .bind(user.id)
        .fetch_one(&state.pool)
        .await?;

    let interests: Vec<String> = sqlx::query_scalar(
        r#"
        SELECT i.name
        FROM interests i
        JOIN user_interests ui ON ui.interest_id = i.id
        WHERE ui.user_id = ?1
        ORDER BY i.name
        "#,
    )
    .bind(updated.id)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(UserResponse::from_user(updated, interests)).into_response())
}
