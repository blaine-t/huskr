use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use oauth2::{
    basic::BasicClient, AuthType, AuthUrl, ClientId, ClientSecret, CsrfToken,
    EndpointNotSet, EndpointSet, PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, Scope,
};
use serde::Deserialize;
use sqlx::SqlitePool;

use crate::{
    auth::Credentials,
    error::AppError,
    models::{IdTokenClaims, User},
};
use axum_login::{AuthnBackend, UserId};

/// oauth2 client with only the auth endpoint configured (we do token exchange via reqwest).
type MsOAuthClient = BasicClient<
    EndpointSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointNotSet,
>;

#[derive(Clone, Debug)]
pub struct MicrosoftBackend {
    pool: SqlitePool,
    oauth_client: MsOAuthClient,
    http: reqwest::Client,
    client_id: String,
    client_secret: String,
    token_url: String,
    redirect_url: String,
}

impl MicrosoftBackend {
    pub fn new(
        pool: SqlitePool,
        client_id: String,
        client_secret: String,
        tenant: &str,
        redirect_url: String,
    ) -> Result<Self, AppError> {
        let auth_url = AuthUrl::new(format!(
            "https://login.microsoftonline.com/{tenant}/oauth2/v2.0/authorize"
        ))
        .map_err(|e| AppError::OAuth(e.to_string()))?;

        let token_url = format!(
            "https://login.microsoftonline.com/{tenant}/oauth2/v2.0/token"
        );

        let redirect = RedirectUrl::new(redirect_url.clone())
            .map_err(|e| AppError::OAuth(e.to_string()))?;

        let oauth_client = BasicClient::new(ClientId::new(client_id.clone()))
            .set_client_secret(ClientSecret::new(client_secret.clone()))
            .set_auth_uri(auth_url)
            .set_auth_type(AuthType::RequestBody)
            .set_redirect_uri(redirect);

        let http = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|e| AppError::Internal(e.to_string()))?;

        Ok(Self {
            pool,
            oauth_client,
            http,
            client_id,
            client_secret,
            token_url,
            redirect_url,
        })
    }

    pub fn authorize_url(&self) -> (oauth2::url::Url, CsrfToken, PkceCodeVerifier) {
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
        let (url, csrf) = self
            .oauth_client
            .authorize_url(CsrfToken::new_random)
            .add_scope(Scope::new("openid".into()))
            .add_scope(Scope::new("email".into()))
            .add_scope(Scope::new("profile".into()))
            .add_scope(Scope::new("offline_access".into()))
            .set_pkce_challenge(pkce_challenge)
            .url();
        (url, csrf, pkce_verifier)
    }
}

#[derive(Debug, Deserialize)]
struct MsTokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    id_token: Option<String>,
}

impl AuthnBackend for MicrosoftBackend {
    type User = User;
    type Credentials = Credentials;
    type Error = AppError;

    async fn authenticate(
        &self,
        creds: Self::Credentials,
    ) -> Result<Option<Self::User>, Self::Error> {
        let token: MsTokenResponse = self
            .http
            .post(&self.token_url)
            .form(&[
                ("client_id", self.client_id.as_str()),
                ("client_secret", self.client_secret.as_str()),
                ("code", creds.code.as_str()),
                ("code_verifier", creds.pkce_verifier.as_str()),
                ("redirect_uri", self.redirect_url.as_str()),
                ("grant_type", "authorization_code"),
            ])
            .send()
            .await
            .map_err(|e| AppError::OAuth(e.to_string()))?
            .json()
            .await
            .map_err(|e| AppError::OAuth(e.to_string()))?;

        let id_token = token
            .id_token
            .as_deref()
            .ok_or_else(|| AppError::OAuth("missing id_token".into()))?;

        let claims: IdTokenClaims = {
            let segment = id_token
                .split('.')
                .nth(1)
                .ok_or_else(|| AppError::OAuth("malformed id_token".into()))?;
            let bytes = URL_SAFE_NO_PAD
                .decode(segment)
                .map_err(|e| AppError::OAuth(e.to_string()))?;
            serde_json::from_slice(&bytes).map_err(|e| AppError::OAuth(e.to_string()))?
        };

        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (oid, email, display_name, tenant_id, access_token, refresh_token, id_token, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, datetime('now'))
            ON CONFLICT(oid) DO UPDATE SET
                email         = excluded.email,
                display_name  = excluded.display_name,
                tenant_id     = excluded.tenant_id,
                access_token  = excluded.access_token,
                refresh_token = excluded.refresh_token,
                id_token      = excluded.id_token,
                updated_at    = excluded.updated_at
            RETURNING *
            "#,
        )
        .bind(&claims.oid)
        .bind(&claims.email)
        .bind(&claims.name)
        .bind(&claims.tid)
        .bind(&token.access_token)
        .bind(&token.refresh_token)
        .bind(id_token)
        .fetch_one(&self.pool)
        .await?;

        Ok(Some(user))
    }

    async fn get_user(&self, user_id: &UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?1")
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(user)
    }
}
