use axum_login::AuthUser;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    pub id: i64,
    pub oid: String,
    pub email: Option<String>,
    pub display_name: Option<String>,
    pub tenant_id: Option<String>,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub id_token: Option<String>,
    // Profile fields
    pub full_name: Option<String>,
    pub age: Option<i64>,
    pub is_rso: bool,
    pub major: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Public-facing user representation sent to the frontend.
/// Deliberately omits all OAuth tokens.
#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: i64,
    pub oid: String,
    pub email: Option<String>,
    pub display_name: Option<String>,
    pub tenant_id: Option<String>,
    // Profile fields
    pub full_name: Option<String>,
    pub age: Option<i64>,
    pub is_rso: bool,
    pub major: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<User> for UserResponse {
    fn from(u: User) -> Self {
        Self {
            id: u.id,
            oid: u.oid,
            email: u.email,
            display_name: u.display_name,
            tenant_id: u.tenant_id,
            full_name: u.full_name,
            age: u.age,
            is_rso: u.is_rso,
            major: u.major,
            created_at: u.created_at,
            updated_at: u.updated_at,
        }
    }
}

impl AuthUser for User {
    type Id = i64;

    fn id(&self) -> Self::Id {
        self.id
    }

    fn session_auth_hash(&self) -> &[u8] {
        match &self.access_token {
            Some(token) => token.as_bytes(),
            None => &[],
        }
    }
}

/// Claims decoded from the Microsoft id_token (JWT middle segment).
#[derive(Debug, Deserialize)]
pub struct IdTokenClaims {
    /// Object ID — stable per-user identifier in AAD
    pub oid: String,
    pub email: Option<String>,
    /// Display name
    pub name: Option<String>,
    /// Tenant ID
    pub tid: Option<String>,
}

// ---------------------------------------------------------------------------
// Interests
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Interest {
    pub id: i64,
    pub name: String,
}

/// Row from the user_interests join table.
#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct UserInterest {
    pub user_id: i64,
    pub interest_id: i64,
}

// ---------------------------------------------------------------------------
// Matches
// ---------------------------------------------------------------------------

/// A match between two users. `user1_id` is always < `user2_id` (enforced
/// by the DB CHECK constraint) to prevent duplicate pairs.
#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Match {
    pub id: i64,
    pub user1_id: i64,
    pub user2_id: i64,
    pub created_at: String,
}

// ---------------------------------------------------------------------------
// Messages
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Message {
    pub id: i64,
    pub sender_id: i64,
    pub recipient_id: i64,
    pub content: String,
    pub created_at: String,
}

/// Payload used when creating a new message.
#[derive(Debug, Deserialize)]
pub struct NewMessage {
    pub recipient_id: i64,
    pub content: String,
}
