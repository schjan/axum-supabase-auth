mod client;
mod errors;
mod types;

use super::types::*;
pub use client::*;
pub use errors::*;
use oauth2::{PkceCodeChallenge, PkceCodeVerifier};
use std::future::Future;
pub use types::*;
use url::Url;

pub trait Api {
    /// Signs up a new user.
    ///
    /// # Arguments
    ///
    /// * `email_or_phone` - The user's email or phone number.
    /// * `password` - The user's password.
    ///
    /// # Returns
    ///
    /// A `SignUpResponse` which may contain either a `User` or a `Session`, depending on the server configuration.
    fn sign_up(
        &self,
        email_or_phone: EmailOrPhone,
        password: impl AsRef<str> + Sized + Send,
    ) -> impl Future<Output = Result<SignUpResponse, ApiError>> + Send;

    fn sign_in(
        &self,
        email_or_phone: EmailOrPhone,
        password: impl AsRef<str> + Send,
    ) -> impl Future<Output = Result<Session, ApiError>>;

    fn logout(&self, access_token: impl AsRef<str>) -> impl Future<Output = Result<(), ApiError>>;

    fn get_user(
        &self,
        access_token: impl AsRef<str>,
    ) -> impl Future<Output = Result<User, ApiError>>;

    fn health_check(&self) -> impl Future<Output = Result<HealthCheckResponse, ApiError>>;

    fn refresh_access_token(
        &self,
        refresh_token: impl AsRef<str>,
    ) -> impl Future<Output = Result<Session, ApiError>>;

    fn list_users(
        &self,
        access_token: impl AsRef<str> + Send,
    ) -> impl Future<Output = Result<UserList, ApiError>> {
        self.list_users_query(access_token, &[])
    }

    fn list_users_query(
        &self,
        access_token: impl AsRef<str> + Send,
        query: &[(&str, &str)],
    ) -> impl Future<Output = Result<UserList, ApiError>>;

    fn create_pkce_oauth_url(&self, req: OAuthRequest, challenge: PkceCodeChallenge) -> Url;

    fn exchange_code_for_session(
        &self,
        code: &str,
        verifier: &PkceCodeVerifier,
    ) -> impl Future<Output = Result<Session, ApiError>>;
}
