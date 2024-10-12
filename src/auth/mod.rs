use crate::api::SignUpResponse;
use crate::{AccessToken, EmailOrPhone, OAuthRequest, OAuthResponse, Session, User};
use std::future::Future;
use thiserror::Error;

pub mod api;
pub mod service;
pub mod types;

pub trait Auth: Clone + Send + Sync + 'static {
    fn sign_up(
        &self,
        email_or_phone: EmailOrPhone,
        password: impl AsRef<str> + Send,
    ) -> impl Future<Output = Result<SignUpResponse, ClientError>> + Send;

    fn sign_in(
        &self,
        email_or_phone: EmailOrPhone,
        password: impl AsRef<str> + Send,
    ) -> impl Future<Output = Result<Session, ClientError>> + Send;

    fn exchange_code_for_session(
        &self,
        code: &str,
        csrf_token_b64: &str,
    ) -> impl Future<Output = Result<Session, ClientError>> + Send;

    // TODO: move to axum?
    fn create_oauth_url(&self, req: OAuthRequest) -> Result<OAuthResponse, ClientError>;

    fn with_token(&self, access_token: AccessToken) -> impl SessionAuth;

    fn with_refresh_token(
        &self,
        access_token: AccessToken,
        refresh_token: String,
    ) -> impl SessionAuth;
}

pub trait SessionAuth {
    fn logout(&self) -> impl Future<Output = Result<(), ClientError>> + Send;

    fn list_users(&self) -> impl Future<Output = Result<Vec<User>, ClientError>> + Send;

    fn refresh(&mut self) -> impl Future<Output = Result<Session, ClientError>> + Send;
}

#[derive(Debug, Error)]
pub enum ClientError {
    #[error("User already signed up")]
    AlreadySignedUp,
    #[error("Wrong credentials")]
    WrongCredentials,
    #[error("User not found")]
    UserNotFound,
    #[error("User not authenticated")]
    NotAuthenticated,
    #[error("Missing refresh token")]
    MissingRefreshToken,
    #[error("Wrong token")]
    WrongToken,
    #[error("GoTrue Internal error")]
    InternalError,
}
