use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use axum_login::AuthSession;
use sqlx::SqlitePool;

use crate::{
    auth::backend::MicrosoftBackend,
    error::AppError,
    models::{Match, NewLike},
};

/// `POST /api/likes`
///
/// Records a like (or pass) from the authenticated user toward another profile.
/// If both users have liked each other a new entry is inserted into `matches`
/// and the match is returned as JSON with `201 Created`.
/// Otherwise `204 No Content` is returned.
pub async fn submit_like(
    auth_session: AuthSession<MicrosoftBackend>,
    State(pool): State<SqlitePool>,
    Json(payload): Json<NewLike>,
) -> Result<impl IntoResponse, AppError> {
    let user = auth_session.user.ok_or(AppError::Unauthorized)?;
    let liker_id = user.id;
    let liked_id = payload.liked_id;
    let is_like = payload.is_like as i64;

    // Upsert the like/pass record.
    sqlx::query(
        r#"
        INSERT INTO likes (liker_id, liked_id, is_like)
        VALUES (?1, ?2, ?3)
        ON CONFLICT(liker_id, liked_id) DO UPDATE SET is_like = excluded.is_like
        "#,
    )
    .bind(liker_id)
    .bind(liked_id)
    .bind(is_like)
    .execute(&pool)
    .await?;

    // Only attempt to match when the current user actually liked the other.
    if !payload.is_like {
        return Ok(StatusCode::NO_CONTENT.into_response());
    }

    // Check whether the other party has already liked the current user.
    let mutual: i64 = sqlx::query_scalar(
        r#"SELECT COUNT(*) FROM likes WHERE liker_id = ?1 AND liked_id = ?2 AND is_like = 1"#,
    )
    .bind(liked_id)
    .bind(liker_id)
    .fetch_one(&pool)
    .await?;

    if mutual == 0 {
        return Ok(StatusCode::NO_CONTENT.into_response());
    }

    // Canonical ordering: smaller id is always user1.
    let (user1_id, user2_id) = if liker_id < liked_id {
        (liker_id, liked_id)
    } else {
        (liked_id, liker_id)
    };

    // Insert the match; if it already exists return the existing row.
    let new_match = sqlx::query_as::<_, Match>(
        r#"
        INSERT INTO matches (user1_id, user2_id)
        VALUES (?1, ?2)
        ON CONFLICT(user1_id, user2_id) DO UPDATE SET user1_id = user1_id
        RETURNING id, user1_id, user2_id, created_at
        "#,
    )
    .bind(user1_id)
    .bind(user2_id)
    .fetch_one(&pool)
    .await?;

    Ok((StatusCode::CREATED, Json(new_match)).into_response())
}
