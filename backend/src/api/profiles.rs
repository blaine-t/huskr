use std::collections::HashMap;

use axum::{
    Json,
    body::Body,
    extract::{Path, State},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use axum_login::AuthSession;
use object_store::{ObjectStoreExt, path::Path as StorePath};
use sqlx::SqlitePool;

use crate::{AppState, auth::backend::MicrosoftBackend, error::AppError, models::{User, UserResponse}};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Fetch interest names for a slice of user IDs in a single query, returning a
/// map of user_id → Vec<interest_name>.
async fn fetch_interests_for_users(
    pool: &SqlitePool,
    user_ids: &[i64],
) -> Result<HashMap<i64, Vec<String>>, sqlx::Error> {
    if user_ids.is_empty() {
        return Ok(HashMap::new());
    }

    // Build a parameterised IN clause dynamically.
    let placeholders: String = user_ids
        .iter()
        .enumerate()
        .map(|(i, _)| format!("?{}", i + 1))
        .collect::<Vec<_>>()
        .join(", ");

    let sql = format!(
        r#"
        SELECT ui.user_id, i.name
        FROM interests i
        JOIN user_interests ui ON ui.interest_id = i.id
        WHERE ui.user_id IN ({placeholders})
        ORDER BY i.name
        "#,
    );

    let mut q = sqlx::query_as::<_, (i64, String)>(&sql);
    for id in user_ids {
        q = q.bind(id);
    }

    let rows = q.fetch_all(pool).await?;

    let mut map: HashMap<i64, Vec<String>> = HashMap::new();
    for (uid, name) in rows {
        map.entry(uid).or_default().push(name);
    }
    Ok(map)
}

// ---------------------------------------------------------------------------
// GET /api/profiles/:id
// ---------------------------------------------------------------------------

/// Returns a single user's public profile (with interests) by their `id`.
/// Returns 404 if the user does not exist.
pub async fn get_profile(
    _auth_session: AuthSession<MicrosoftBackend>,
    State(pool): State<SqlitePool>,
    Path(profile_id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?1")
        .bind(profile_id)
        .fetch_optional(&pool)
        .await?;

    let Some(user) = user else {
        return Ok(StatusCode::NOT_FOUND.into_response());
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
// GET /api/profiles/compatible
// ---------------------------------------------------------------------------

/// Compatibility scoring (higher = better match):
///   +3 per shared interest
///   +2 for the same major
///   +1 for the same RSO status
///
/// Users the current user has already liked/passed are excluded.
/// Results are sorted by score descending and capped at 50.
pub async fn compatible_profiles(
    auth_session: AuthSession<MicrosoftBackend>,
    State(pool): State<SqlitePool>,
) -> Result<impl IntoResponse, AppError> {
    let me = auth_session.user.ok_or(AppError::Unauthorized)?;

    // Score every user the current user has not yet acted on.
    // The subqueries reference ?1 (current user id) for scoring and exclusion.
    let scored_users = sqlx::query_as::<_, User>(
        r#"
        SELECT u.*
        FROM users u
        WHERE u.id != ?1
          AND u.id NOT IN (
              SELECT liked_id FROM likes WHERE liker_id = ?1
          )
        ORDER BY
            -- shared interests (×3)
            (
                SELECT COUNT(*) * 3
                FROM user_interests a
                JOIN user_interests b ON a.interest_id = b.interest_id
                WHERE a.user_id = ?1 AND b.user_id = u.id
            )
            -- same major (×2)
            + CASE
                WHEN u.major IS NOT NULL
                 AND u.major = (SELECT major FROM users WHERE id = ?1)
                THEN 2 ELSE 0
              END
            -- same RSO status (×1)
            + CASE
                WHEN u.is_rso = (SELECT is_rso FROM users WHERE id = ?1)
                THEN 1 ELSE 0
              END
            DESC
        LIMIT 50
        "#,
    )
    .bind(me.id)
    .fetch_all(&pool)
    .await?;

    let ids: Vec<i64> = scored_users.iter().map(|u| u.id).collect();
    let mut interest_map = fetch_interests_for_users(&pool, &ids).await?;

    let profiles: Vec<UserResponse> = scored_users
        .into_iter()
        .map(|u| {
            let interests = interest_map.remove(&u.id).unwrap_or_default();
            UserResponse::from_user(u, interests)
        })
        .collect();

    Ok(Json(profiles).into_response())
}

// ---------------------------------------------------------------------------
// GET /profiles/:id/image
// ---------------------------------------------------------------------------

/// Streams the profile image for user `id` from object storage.
/// Returns 404 if the user has no image on file.
pub async fn get_profile_image(
    _auth_session: AuthSession<MicrosoftBackend>,
    State(state): State<AppState>,
    Path(profile_id): Path<i64>,
) -> Result<Response, AppError> {
    let image_key: Option<String> = sqlx::query_scalar(
        "SELECT image_key FROM users WHERE id = ?1",
    )
    .bind(profile_id)
    .fetch_optional(&state.pool)
    .await?
    .flatten();

    let Some(key) = image_key else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };

    let path = StorePath::from(key.as_str());
    let result = state
        .store
        .get(&path)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let bytes = result
        .bytes()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    Ok((
        [(header::CONTENT_TYPE, "image/jpeg")],
        Body::from(bytes),
    )
        .into_response())
}
