use axum::{extract::State, response::IntoResponse, Json};
use axum_login::AuthSession;
use serde::Serialize;
use sqlx::SqlitePool;

use crate::{
    auth::backend::MicrosoftBackend,
    error::AppError,
    models::{Match, User, UserResponse},
};

#[derive(Debug, Serialize)]
pub struct MatchResponse {
    pub id: i64,
    pub user: UserResponse,
    pub created_at: String,
}

/// `GET /matches`
///
/// Returns all matches for the authenticated user, each with the other user's
/// public profile (including interests).
pub async fn get_matches(
    auth_session: AuthSession<MicrosoftBackend>,
    State(pool): State<SqlitePool>,
) -> Result<impl IntoResponse, AppError> {
    let me = auth_session.user.ok_or(AppError::Unauthorized)?;

    let matches = sqlx::query_as::<_, Match>(
        r#"
        SELECT id, user1_id, user2_id, created_at
        FROM matches
        WHERE user1_id = ?1 OR user2_id = ?1
        ORDER BY created_at DESC
        "#,
    )
    .bind(me.id)
    .fetch_all(&pool)
    .await?;

    let mut result = Vec::with_capacity(matches.len());

    for m in matches {
        let other_id = if m.user1_id == me.id {
            m.user2_id
        } else {
            m.user1_id
        };

        let other = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?1")
            .bind(other_id)
            .fetch_one(&pool)
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
        .bind(other_id)
        .fetch_all(&pool)
        .await?;

        result.push(MatchResponse {
            id: m.id,
            user: UserResponse::from_user(other, interests),
            created_at: m.created_at,
        });
    }

    Ok(Json(result))
}
