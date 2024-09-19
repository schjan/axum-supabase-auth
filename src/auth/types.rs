use serde::Deserialize;
use std::fmt::{Debug, Formatter};

#[derive(Clone, Deserialize)]
pub struct Session {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i32,
    pub refresh_token: String,
    pub user: User,
}

impl Debug for Session {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Session")
            .field("access_token", &"[redacted]")
            .field("token_type", &self.token_type)
            .field("expires_in", &self.expires_in)
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

#[derive(Debug, Clone, Deserialize)]
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