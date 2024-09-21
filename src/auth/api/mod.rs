mod types;

use super::types::*;
use crate::api::types::HealthCheckResponse;
use bon::bon;
use either::Either;
use oauth2::{PkceCodeChallenge, PkceCodeVerifier};
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{Client, Method, StatusCode, Url};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;
use thiserror::Error;
use tracing::{error, instrument, warn};

#[derive(Clone)]
pub struct Api {
    url: Url,
    client: Client,
    headers: Arc<HeaderMap>,
}

#[bon]
impl Api {
    #[builder(finish_fn=send)]
    async fn send_request<T, B>(
        &self,
        #[builder(start_fn)] method: Method,
        #[builder(start_fn)] endpoint: &str,
        query: Option<&[(&str, &str)]>,
        body: Option<&B>,
        access_token: Option<&str>,
    ) -> Result<T, ApiError>
    where
        T: DeserializeOwned,
        B: Serialize + ?Sized,
    {
        let url = self.url.join(endpoint)?;

        let mut request = self
            .client
            .request(method.clone(), url)
            .headers((*self.headers).clone());

        if let Some(q) = query {
            if !q.is_empty() {
                request = request.query(q);
            }
        }

        if let Some(b) = body {
            request = request.json(b);
        }

        if let Some(token) = access_token {
            request = request.bearer_auth(token);
        }

        let response = request.send().await?;

        match response.error_for_status_ref() {
            Ok(_) => {}
            Err(err) => {
                return if let Some(status) = err.status() {
                    let body: ApiErrorResponse = response.json().await.unwrap_or_default();

                    warn!(%err,
                        method = %method,
                        endpoint = %endpoint,
                        body = ?body,
                        status = status.as_u16(),
                        "Request failed");

                    Err(ApiError::HttpError(err, status))
                } else {
                    Err(ApiError::Unknown(err))
                }
            }
        };

        let result = response.json::<T>().await?;
        Ok(result)
    }
}

impl Api {
    pub fn new(url: Url, timeout: Duration, api_key: &str) -> Self {
        let client = Client::builder()
            .timeout(timeout)
            .user_agent("portal")
            .build()
            .unwrap();

        let mut headers = HeaderMap::new();
        headers.insert("apiKey", HeaderValue::from_str(api_key).unwrap());
        let headers = Arc::new(headers);

        Self {
            url,
            client,
            headers,
        }
    }

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
    #[instrument(skip(self, password), fields(user_id))]
    pub async fn sign_up(
        &self,
        email_or_phone: EmailOrPhone,
        password: impl AsRef<str> + Sized,
    ) -> Result<SignUpResponse, ApiError> {
        self.send_request(Method::POST, "signup")
            .body(&self.sign_in_up_body(&email_or_phone, &password))
            .send()
            .await
    }

    #[instrument(skip(self, password))]
    pub async fn sign_in(
        &self,
        email_or_phone: EmailOrPhone,
        password: impl AsRef<str>,
    ) -> Result<Session, ApiError> {
        self.send_request(Method::POST, "token")
            .query(&[("grant_type", "password")])
            .body(&self.sign_in_up_body(&email_or_phone, &password))
            .send()
            .await
    }

    fn sign_in_up_body<'a>(
        &'a self,
        email_or_phone: &'a EmailOrPhone,
        password: &'a impl AsRef<str>,
    ) -> SignInUpBody<'a> {
        match email_or_phone {
            EmailOrPhone::Email(email) => SignInUpBody {
                email: Some(email),
                phone: None,
                password: password.as_ref(),
            },
            EmailOrPhone::Phone(phone) => SignInUpBody {
                email: None,
                phone: Some(phone.as_str()),
                password: password.as_ref(),
            },
        }
    }

    #[instrument(skip(self, access_token))]
    pub async fn logout(&self, access_token: impl AsRef<str>) -> Result<(), ApiError> {
        self.send_request::<(), ()>(Method::POST, "logout")
            .access_token(access_token.as_ref())
            .send()
            .await
    }

    #[instrument(skip(self, access_token))]
    pub async fn get_user(&self, access_token: impl AsRef<str>) -> Result<User, ApiError> {
        self.send_request::<_, ()>(Method::GET, "user")
            .access_token(access_token.as_ref())
            .send()
            .await
    }

    #[instrument(skip(self))]
    pub async fn health_check(&self) -> Result<HealthCheckResponse, ApiError> {
        self.send_request::<_, ()>(Method::GET, "health")
            .send()
            .await
    }

    #[instrument(skip(self, refresh_token))]
    pub async fn refresh_access_token(
        &self,
        refresh_token: impl AsRef<str>,
    ) -> Result<Session, ApiError> {
        self.send_request(Method::POST, "token")
            .query(&[("grant_type", "refresh_token")])
            .body(&json!({
                "refresh_token": refresh_token.as_ref(),
            }))
            .send()
            .await
    }

    pub async fn list_users(&self, access_token: impl AsRef<str>) -> Result<UserList, ApiError> {
        self.list_users_query(access_token, &[]).await
    }

    #[instrument(skip(self, access_token, query))]
    pub async fn list_users_query(
        &self,
        access_token: impl AsRef<str>,
        query: &[(&str, &str)],
    ) -> Result<UserList, ApiError> {
        self.send_request::<_, ()>(Method::GET, "admin/users")
            .query(query)
            .access_token(access_token.as_ref())
            .send()
            .await
    }

    pub fn create_pkce_oauth_url(&self, req: OAuthRequest, challenge: PkceCodeChallenge) -> Url {
        let query = format!(
            "provider={}&redirect_to={}&code_challenge={}&code_challenge_method={}",
            req.provider,
            req.redirect_to,
            challenge.as_str(),
            challenge.method().as_str()
        );

        let mut endpoint = self.url.join("authorize").unwrap();
        endpoint.set_query(Some(&query));

        endpoint
    }

    pub async fn exchange_code_for_session(
        &self,
        code: &str,
        verifier: &PkceCodeVerifier,
    ) -> Result<Session, ApiError> {
        self.send_request(Method::POST, "token")
            .query(&[("grant_type", "pkce")])
            .body(&json!({
                "auth_code": code,
                "code_verifier": verifier.secret(),
            }))
            .send()
            .await
    }
}

#[derive(Serialize)]
struct SignInUpBody<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    email: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    phone: Option<&'a str>,
    password: &'a str,
}

#[derive(Deserialize, Debug)]
#[serde(transparent)]
pub struct SignUpResponse {
    #[serde(with = "either::serde_untagged")]
    inner: Either<User, Session>,
}

impl SignUpResponse {
    pub fn session(self) -> Option<Session> {
        self.into()
    }

    pub fn user(self) -> Option<User> {
        self.into()
    }
}

impl AsRef<User> for SignUpResponse {
    fn as_ref(&self) -> &User {
        match self.inner {
            Either::Left(ref user) => user,
            Either::Right(ref session) => &session.user,
        }
    }
}

impl From<SignUpResponse> for Option<User> {
    fn from(val: SignUpResponse) -> Self {
        val.inner.left()
    }
}

impl From<SignUpResponse> for Option<Session> {
    fn from(val: SignUpResponse) -> Self {
        val.inner.right()
    }
}

/// https://github.com/supabase/auth/blob/master/internal/api/errors.go
#[derive(Debug, Error)]
pub enum ApiError {
    #[error("Http error with status {1}: {0}")]
    HttpError(#[source] reqwest::Error, StatusCode),
    #[error(transparent)]
    Unknown(#[from] reqwest::Error),
    #[error("URL parsing error: {0}")]
    UrlError(#[from] url::ParseError),
}

#[derive(Debug, Deserialize, Default)]
pub struct ApiErrorResponse {
    pub code: u16,
    pub error_code: ApiErrorCode,
    pub msg: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApiErrorCode {
    Unknown,
    SignupDisabled,
    UserAlreadyExists,
}

impl Default for ApiErrorCode {
    fn default() -> Self {
        Self::Unknown
    }
}
