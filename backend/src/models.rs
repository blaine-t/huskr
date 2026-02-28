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
