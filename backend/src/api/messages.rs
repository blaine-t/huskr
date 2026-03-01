use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use axum_login::AuthSession;
use sqlx::SqlitePool;

use crate::{
    auth::backend::MicrosoftBackend,
    error::AppError,
    models::{Message, NewMessage},
};

/// `GET /api/messages/:user_id`
///
/// Retrieves all messages between the authenticated user and the specified user,
/// ordered by creation time (oldest first).
pub async fn get_messages(
    auth_session: AuthSession<MicrosoftBackend>,
    State(pool): State<SqlitePool>,
    Path(user_id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    let user = auth_session.user.ok_or(AppError::Unauthorized)?;
    let current_user_id = user.id;

    // Fetch all messages where either:
    // - current user sent to specified user, OR
    // - specified user sent to current user
    let messages = sqlx::query_as::<_, Message>(
        r#"
        SELECT id, sender_id, recipient_id, content, created_at
        FROM messages
        WHERE (sender_id = ?1 AND recipient_id = ?2)
           OR (sender_id = ?2 AND recipient_id = ?1)
        ORDER BY created_at ASC
        "#,
    )
    .bind(current_user_id)
    .bind(user_id)
    .fetch_all(&pool)
    .await?;

    Ok(Json(messages))
}

/// `POST /api/message`
///
/// Sends a new message from the authenticated user to the specified recipient.
/// Returns the created message with status code 201.
pub async fn send_message(
    auth_session: AuthSession<MicrosoftBackend>,
    State(pool): State<SqlitePool>,
    Json(payload): Json<NewMessage>,
) -> Result<impl IntoResponse, AppError> {
    let user = auth_session.user.ok_or(AppError::Unauthorized)?;
    let sender_id = user.id;

    // Insert the new message and return the created record
    let message = sqlx::query_as::<_, Message>(
        r#"
        INSERT INTO messages (sender_id, recipient_id, content)
        VALUES (?1, ?2, ?3)
        RETURNING id, sender_id, recipient_id, content, created_at
        "#,
    )
    .bind(sender_id)
    .bind(payload.recipient_id)
    .bind(&payload.content)
    .fetch_one(&pool)
    .await?;

    Ok((StatusCode::CREATED, Json(message)))
}
