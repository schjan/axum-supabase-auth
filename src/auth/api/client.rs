use crate::api::types::HealthCheckResponse;
use crate::api::{
    Api, ApiError, ApiErrorResponse, IntoApi, OAuthErrorResponse, SignInUpBody, SignUpResponse,
};
use crate::{EmailOrPhone, OAuthRequest, Session, User, UserList};
use axum::http::{HeaderMap, HeaderValue, Method};
use bon::bon;
use oauth2::{PkceCodeChallenge, PkceCodeVerifier};
use reqwest::{Client, Response};
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::json;
use std::fmt::Debug;
use std::sync::Arc;
use std::time::Duration;
use tracing::{instrument, trace, warn};
use url::Url;

#[derive(Clone)]
pub struct ApiClient {
    url: Url,
    client: Client,
    headers: Arc<HeaderMap>,
}

#[bon]
impl ApiClient {
    #[instrument(
        name = "api_request",
        skip(self, body, access_token, query),
        fields(status)
    )]
    #[builder(finish_fn=send)]
    async fn send_request<T, B, E>(
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
        E: DeserializeOwned + IntoApi + Debug,
    {
        let url = self.url.join(endpoint)?;

        let mut request = self
            .client
            .request(method, url)
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

        request.send().await?.handle_response::<_, E>().await
    }
}

impl ApiClient {
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
}

impl Api for ApiClient {
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
    async fn sign_up(
        &self,
        email_or_phone: EmailOrPhone,
        password: impl AsRef<str> + Sized + Send,
    ) -> Result<SignUpResponse, ApiError> {
        self.send_request::<_, _, ApiErrorResponse>(Method::POST, "signup")
            .body(&self.sign_in_up_body(&email_or_phone, &password))
            .send()
            .await
    }

    #[instrument(skip(self, password))]
    async fn sign_in(
        &self,
        email_or_phone: EmailOrPhone,
        password: impl AsRef<str>,
    ) -> Result<Session, ApiError> {
        self.send_request::<_, _, ApiErrorResponse>(Method::POST, "token")
            .query(&[("grant_type", "password")])
            .body(&self.sign_in_up_body(&email_or_phone, &password))
            .send()
            .await
    }

    #[instrument(skip(self, access_token))]
    async fn logout(&self, access_token: impl AsRef<str>) -> Result<(), ApiError> {
        let endpoint = self.url.join("logout")?;

        self.client
            .post(endpoint)
            .headers((*self.headers).clone())
            .bearer_auth(access_token.as_ref())
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }

    #[instrument(skip(self, access_token))]
    async fn get_user(&self, access_token: impl AsRef<str>) -> Result<User, ApiError> {
        self.send_request::<_, (), ApiErrorResponse>(Method::GET, "user")
            .access_token(access_token.as_ref())
            .send()
            .await
    }

    #[instrument(skip(self))]
    async fn health_check(&self) -> Result<HealthCheckResponse, ApiError> {
        self.send_request::<_, (), ApiErrorResponse>(Method::GET, "health")
            .send()
            .await
    }

    #[instrument(skip(self, refresh_token))]
    async fn refresh_access_token(
        &self,
        refresh_token: impl AsRef<str>,
    ) -> Result<Session, ApiError> {
        self.send_request::<_, _, OAuthErrorResponse>(Method::POST, "token")
            .query(&[("grant_type", "refresh_token")])
            .body(&json!({
                "refresh_token": refresh_token.as_ref(),
            }))
            .send()
            .await
    }

    #[instrument(skip(self, access_token, query))]
    async fn list_users_query(
        &self,
        access_token: impl AsRef<str>,
        query: &[(&str, &str)],
    ) -> Result<UserList, ApiError> {
        self.send_request::<_, (), ApiErrorResponse>(Method::GET, "admin/users")
            .query(query)
            .access_token(access_token.as_ref())
            .send()
            .await
    }

    fn create_pkce_oauth_url(&self, req: OAuthRequest, challenge: PkceCodeChallenge) -> Url {
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

    async fn exchange_code_for_session(
        &self,
        code: &str,
        verifier: &PkceCodeVerifier,
    ) -> Result<Session, ApiError> {
        self.send_request::<_, _, ApiErrorResponse>(Method::POST, "token")
            .query(&[("grant_type", "pkce")])
            .body(&json!({
                "auth_code": code,
                "code_verifier": verifier.secret(),
            }))
            .send()
            .await
    }
}

trait HandleApiResponse {
    async fn handle_response<T, E>(self) -> Result<T, ApiError>
    where
        T: DeserializeOwned,
        E: DeserializeOwned + IntoApi + Debug;
}

impl HandleApiResponse for Response {
    async fn handle_response<T, E>(self) -> Result<T, ApiError>
    where
        T: DeserializeOwned,
        E: DeserializeOwned + IntoApi + Debug,
    {
        let status = self.status();
        tracing::Span::current().record("status", status.as_u16());

        match self.error_for_status_ref() {
            Ok(_) => Ok(self.json::<T>().await?),
            Err(err) => {
                if let Ok(body) = self.json::<E>().await {
                    trace!(%err, body = ?body, "Request failed");
                    Err(body.with_status(status))
                } else {
                    warn!(%err, "Request failed with unhandled HTTP error");
                    Err(ApiError::UnknownHTTP(err))
                }
            }
        }
    }
}
