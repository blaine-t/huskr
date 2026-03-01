use axum::{
    body::Body,
    extract::{Multipart, Path, State},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
    Json,
};
use axum_login::AuthSession;
use object_store::{ObjectStoreExt, PutPayload, path::Path as StorePath};

use crate::{
    AppState,
    auth::backend::MicrosoftBackend,
    error::AppError,
    models::Message,
};

/// `GET /messages/:user_id`
///
/// Retrieves all messages between the authenticated user and the specified user,
/// ordered by creation time (oldest first).
pub async fn get_messages(
    auth_session: AuthSession<MicrosoftBackend>,
    State(state): State<AppState>,
    Path(user_id): Path<i64>,
) -> Result<impl IntoResponse, AppError> {
    let user = auth_session.user.ok_or(AppError::Unauthorized)?;
    let current_user_id = user.id;

    let messages = sqlx::query_as::<_, Message>(
        r#"
        SELECT id, sender_id, recipient_id, content, image_key, created_at
        FROM messages
        WHERE (sender_id = ?1 AND recipient_id = ?2)
           OR (sender_id = ?2 AND recipient_id = ?1)
        ORDER BY created_at ASC
        "#,
    )
    .bind(current_user_id)
    .bind(user_id)
    .fetch_all(&state.pool)
    .await?;

    Ok(Json(messages))
}

/// `POST /message`
///
/// Accepts multipart/form-data:
///   - `recipient_id` — text, required
///   - `content`      — text, optional
///   - `image`        — file, optional
///
/// Returns the created message with status 201.
pub async fn send_message(
    auth_session: AuthSession<MicrosoftBackend>,
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    let user = auth_session.user.ok_or(AppError::Unauthorized)?;
    let sender_id = user.id;

    let mut recipient_id: Option<i64> = None;
    let mut content = String::new();
    let mut image_data: Option<bytes::Bytes> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
    {
        match field.name().unwrap_or("") {
            "recipient_id" => {
                let text = field
                    .text()
                    .await
                    .map_err(|e| AppError::Internal(e.to_string()))?;
                recipient_id = text.trim().parse::<i64>().ok();
            }
            "content" => {
                content = field
                    .text()
                    .await
                    .map_err(|e| AppError::Internal(e.to_string()))?;
            }
            "image" => {
                let data = field
                    .bytes()
                    .await
                    .map_err(|e| AppError::Internal(e.to_string()))?;
                if !data.is_empty() {
                    image_data = Some(data);
                }
            }
            _ => {}
        }
    }

    let recipient_id = recipient_id.ok_or_else(|| AppError::Internal("missing recipient_id".into()))?;

    // Insert message row first (image_key is filled in after we know the id)
    let message = sqlx::query_as::<_, Message>(
        r#"
        INSERT INTO messages (sender_id, recipient_id, content)
        VALUES (?1, ?2, ?3)
        RETURNING id, sender_id, recipient_id, content, image_key, created_at
        "#,
    )
    .bind(sender_id)
    .bind(recipient_id)
    .bind(&content)
    .fetch_one(&state.pool)
    .await?;

    // If an image was uploaded, store it and update image_key
    let message = if let Some(data) = image_data {
        let key = format!("messages/{}", message.id);
        let path = StorePath::from(key.as_str());
        state
            .store
            .put(&path, PutPayload::from(data))
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        sqlx::query(
            "UPDATE messages SET image_key = ?1 WHERE id = ?2",
        )
        .bind(&key)
        .bind(message.id)
        .execute(&state.pool)
        .await?;

        sqlx::query_as::<_, Message>(
            "SELECT id, sender_id, recipient_id, content, image_key, created_at FROM messages WHERE id = ?1",
        )
        .bind(message.id)
        .fetch_one(&state.pool)
        .await?
    } else {
        message
    };

    Ok((StatusCode::CREATED, Json(message)))
}

/// `GET /messages/:message_id/image`
///
/// Streams the image for a message from object storage.
/// Returns 404 if the message has no image.
pub async fn get_message_image(
    _auth_session: AuthSession<MicrosoftBackend>,
    State(state): State<AppState>,
    Path(message_id): Path<i64>,
) -> Result<Response, AppError> {
    let image_key: Option<String> = sqlx::query_scalar(
        "SELECT image_key FROM messages WHERE id = ?1",
    )
    .bind(message_id)
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
