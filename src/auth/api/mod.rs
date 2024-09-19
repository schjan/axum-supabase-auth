use super::types::*;
use either::Either;
use oauth2::{PkceCodeChallenge, PkceCodeVerifier};
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{Client, StatusCode, Url};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fmt::Display;
use std::time::Duration;
use thiserror::Error;
use tracing::{error, instrument, Span};

#[derive(Clone)]
pub struct Api {
    url: Url,
    client: Client,
    headers: HeaderMap,
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

        Self {
            url,
            client,
            headers,
        }
    }

    /// Register a new user.
    /// Returns a Session if autoconfirm is enabled for the instance, else a Session.
    #[instrument(skip(self, password), fields(user_id))]
    pub async fn sign_up(
        &self,
        email_or_phone: EmailOrPhone,
        password: impl AsRef<str> + Sized,
    ) -> Result<SignUpResponse, SignUpError> {
        let endpoint = self.url.join("signup").unwrap();

        let response = self
            .client
            .post(endpoint)
            .headers(self.headers.clone())
            .json(&self.sign_in_up_body(email_or_phone, password))
            .send()
            .await?;

        if response.status() == StatusCode::UNPROCESSABLE_ENTITY {
            return Err(SignUpError::UnableToSignUp);
        };

        let response: SignUpResponse = response.trace_error_for_status().await?.json().await?;

        Ok(response)
    }

    #[instrument(skip(self, password))]
    pub async fn sign_in(
        &self,
        email_or_phone: EmailOrPhone,
        password: impl AsRef<str>,
    ) -> Result<Session, reqwest::Error> {
        let endpoint = self.url.join("token").unwrap();

        let response: Session = self
            .client
            .post(endpoint)
            .headers(self.headers.clone())
            .query(&[("grant_type", "password")])
            .json(&self.sign_in_up_body(email_or_phone, password))
            .send()
            .await?
            .trace_error_for_status()
            .await?
            .json()
            .await?;

        Ok(response)
    }

    fn sign_in_up_body(
        &self,
        email_or_phone: EmailOrPhone,
        password: impl AsRef<str>,
    ) -> serde_json::Value {
        match email_or_phone {
            EmailOrPhone::Email(email) => json!({
                "email": email,
                "password": password.as_ref(),
            }),
            EmailOrPhone::Phone(phone) => json!({
                "phone": phone,
                "password": password.as_ref(),
            }),
        }
    }

    #[instrument(skip(self, access_token))]
    pub async fn logout(&self, access_token: impl Display) -> Result<(), reqwest::Error> {
        let endpoint = self.url.join("logout").unwrap();

        self.client
            .post(endpoint)
            .headers(self.headers.clone())
            .bearer_auth(access_token)
            .send()
            .await?
            .trace_error_for_status()
            .await?;

        Ok(())
    }

    #[instrument(skip(self, access_token))]
    pub async fn get_user(&self, access_token: impl Display) -> Result<User, reqwest::Error> {
        let endpoint = self.url.join("user").unwrap();

        let user = self
            .client
            .get(endpoint)
            .headers(self.headers.clone())
            .bearer_auth(access_token)
            .send()
            .await?
            .trace_error_for_status()
            .await?
            .json()
            .await?;

        Ok(user)
    }

    #[instrument(skip(self, refresh_token))]
    pub async fn refresh_access_token(
        &self,
        refresh_token: impl AsRef<str>,
    ) -> Result<Session, reqwest::Error> {
        let endpoint = self.url.join("token").unwrap();

        let response = self
            .client
            .post(endpoint)
            .headers(self.headers.clone())
            .query(&[("grant_type", "refresh_token")])
            .json(&json!({
                "refresh_token": refresh_token.as_ref(),
            }))
            .send()
            .await?
            .trace_error_for_status()
            .await?
            .json()
            .await?;

        Ok(response)
    }

    pub async fn list_users(&self, access_token: impl Display) -> Result<UserList, reqwest::Error> {
        self.list_users_query(access_token, &{}).await
    }

    #[instrument(skip(self, access_token, query))]
    pub async fn list_users_query(
        &self,
        access_token: impl Display,
        query: &(impl Serialize + ?Sized),
    ) -> Result<UserList, reqwest::Error> {
        let endpoint = self.url.join("admin/users").unwrap();

        let users = self
            .client
            .get(endpoint)
            .headers(self.headers.clone())
            .query(query)
            .bearer_auth(access_token)
            .send()
            .await?
            .trace_error_for_status()
            .await?
            .json()
            .await?;

        Ok(users)
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
    ) -> Result<Session, reqwest::Error> {
        let endpoint = self.url.join("token").unwrap();

        let response: Session = self
            .client
            .post(endpoint)
            .headers(self.headers.clone())
            .query(&[("grant_type", "pkce")])
            .json(&json!({
                "auth_code": code,
                "code_verifier": verifier.secret(),
            }))
            .send()
            .await?
            .trace_error_for_status()
            .await?
            .json()
            .await?;

        Ok(response)
    }
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

#[derive(Debug, Error)]
pub enum SignUpError {
    #[error("Signups not allowed for this instance or user already existing")]
    UnableToSignUp,
    #[error(transparent)]
    Unknown(#[from] reqwest::Error),
}

trait TraceErrorForStatus: Sized {
    async fn trace_error_for_status(self) -> reqwest::Result<Self>;
}

impl TraceErrorForStatus for reqwest::Response {
    async fn trace_error_for_status(self) -> reqwest::Result<Self> {
        match self.error_for_status_ref() {
            Ok(_) => Ok(self),
            Err(error) => {
                if let Some(status) = error.status() {
                    let body: serde_json::Value = self.json().await.unwrap_or_default();
                    error!(%error, status = status.as_str(), body = %body, "Request failed");
                } else {
                    error!(%error, "Request failed");
                }
                Err(error)
            }
        }
    }
}
