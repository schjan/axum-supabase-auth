use serde::Deserialize;
use std::fmt::{Debug, Formatter};
use time::OffsetDateTime;

#[derive(Clone, Deserialize, PartialEq, Eq, Hash)]
pub struct AccessToken(String);

impl Debug for AccessToken {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[redacted]")
    }
}

impl AsRef<str> for AccessToken {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

impl From<String> for AccessToken {
    fn from(token: String) -> Self {
        Self(token)
    }
}

impl From<AccessToken> for String {
    fn from(value: AccessToken) -> Self {
        value.0
    }
}

#[derive(Clone, Deserialize, PartialEq, Eq, Hash)]
pub struct RefreshToken(String);

impl Debug for RefreshToken {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[redacted]")
    }
}

impl AsRef<str> for RefreshToken {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

impl From<String> for RefreshToken {
    fn from(token: String) -> Self {
        Self(token)
    }
}

impl From<RefreshToken> for String {
    fn from(value: RefreshToken) -> Self {
        value.0
    }
}

#[derive(Clone, Deserialize, PartialEq, Eq)]
pub struct Session {
    pub access_token: AccessToken,
    pub token_type: String,
    pub expires_in: i32,
    #[serde(with = "time::serde::timestamp")]
    pub expires_at: OffsetDateTime,
    pub refresh_token: RefreshToken,
    pub user: User,
}

impl Debug for Session {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Session")
            .field("access_token", &"[redacted]")
            .field("token_type", &self.token_type)
            .field("expires_in", &self.expires_in)
            .field("expires_at", &self.expires_at)
            .field("refresh_token", &"[redacted]")
            .field("user", &self.user)
            .finish()
    }
}

#[derive(Debug)]
pub enum EmailOrPhone {
    Email(String),
    Phone(String),
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct User {
    pub id: String,
    pub email: String,
    pub aud: String,
    pub role: String,
    pub email_confirmed_at: Option<String>,
    pub phone: String,
    pub last_sign_in_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug)]
pub struct UserAttributes {
    pub email: String,
    pub password: String,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UserList {
    pub users: Vec<User>,
}

#[derive(Debug, Deserialize)]
pub struct UserUpdate {
    pub id: String,
    pub email: String,
    pub new_email: String,
    pub email_change_sent_at: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug)]
pub struct OAuthRequest {
    pub provider: String,
    pub redirect_to: String,
}

#[derive(Debug)]
pub struct OAuthResponse {
    pub supabase_url: String,
    pub csrf_token: String,
}
