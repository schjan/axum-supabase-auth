use crate::api::{Api, ApiError, ApiErrorCode, SignUpResponse};
use crate::auth::api::ApiClient;
use crate::auth::ClientError;
use crate::{
    AccessToken, Auth, EmailOrPhone, OAuthRequest, OAuthResponse, Session, SessionAuth, User,
};
use axum::http::StatusCode;
use base64::prelude::{Engine as _, BASE64_STANDARD};
use oauth2::{PkceCodeChallenge, PkceCodeVerifier};
use reqwest::Url;
use std::sync::Arc;
use std::time::Duration;
use tracing::error;

#[derive(Clone)]
pub struct AuthService {
    api: Arc<ApiClient>,
}

impl AuthService {
    pub fn new(url: Url, api_key: &str) -> Self {
        Self::new_with_timeout(url, api_key, Duration::from_secs(2))
    }

    pub fn new_with_timeout(url: Url, api_key: &str, timeout: Duration) -> Self {
        Self {
            api: Arc::new(ApiClient::new(url, timeout, api_key)),
        }
    }
}

impl Auth for AuthService {
    async fn sign_up(
        &self,
        email_or_phone: EmailOrPhone,
        password: impl AsRef<str> + Send,
    ) -> Result<SignUpResponse, ClientError> {
        match self.api.sign_up(email_or_phone, password).await {
            Ok(session) => Ok(session),
            Err(ApiError::Request(StatusCode::UNPROCESSABLE_ENTITY, _, _)) => {
                Err(ClientError::AlreadySignedUp)
            }
            Err(e) => {
                error!("Error signing up: {:?}", e);
                Err(ClientError::InternalError)
            }
        }
    }

    async fn sign_in(
        &self,
        email_or_phone: EmailOrPhone,
        password: impl AsRef<str> + Send,
    ) -> Result<Session, ClientError> {
        match self.api.sign_in(email_or_phone, password).await {
            Ok(session) => Ok(session),
            Err(ApiError::Request(StatusCode::BAD_REQUEST, _, _)) => {
                Err(ClientError::WrongCredentials)
            }
            Err(e) => {
                error!("Error signing in: {:?}", e);
                Err(ClientError::InternalError)
            }
        }
    }

    async fn exchange_code_for_session(
        &self,
        code: &str,
        csrf_token_b64: &str,
    ) -> Result<Session, ClientError> {
        let csrf_token = BASE64_STANDARD
            .decode(csrf_token_b64)
            .map_err(|_| ClientError::WrongToken)?;

        let verifier = PkceCodeVerifier::new(
            String::from_utf8(csrf_token).map_err(|_| ClientError::WrongToken)?,
        );

        match self.api.exchange_code_for_session(code, &verifier).await {
            Ok(session) => Ok(session),
            Err(e) => {
                error!("Error exchanging code for session: {:?}", e);
                Err(ClientError::InternalError)
            }
        }
    }

    fn create_oauth_url(&self, req: OAuthRequest) -> Result<OAuthResponse, ClientError> {
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        let url = self.api.create_pkce_oauth_url(req, pkce_challenge);
        let csrf_token = BASE64_STANDARD.encode(pkce_verifier.secret());

        Ok(OAuthResponse {
            supabase_url: url.to_string(),
            csrf_token,
        })
    }

    fn with_token(&self, access_token: AccessToken) -> impl SessionAuth {
        SessionAuthService::with_token(self.clone(), access_token)
    }

    fn with_refresh_token(
        &self,
        access_token: AccessToken,
        refresh_token: String,
    ) -> impl SessionAuth {
        SessionAuthService::with_refresh_token(self.clone(), access_token, refresh_token)
    }
}

#[derive(Clone)]
pub struct SessionAuthService {
    auth: AuthService,
    access_token: AccessToken,
    refresh_token: Option<String>,
}

impl AsRef<AuthService> for SessionAuthService {
    fn as_ref(&self) -> &AuthService {
        &self.auth
    }
}

impl SessionAuthService {
    fn with_token(auth: AuthService, access_token: AccessToken) -> Self {
        Self {
            auth,
            access_token,
            refresh_token: None,
        }
    }

    fn with_refresh_token(
        auth: AuthService,
        access_token: AccessToken,
        refresh_token: String,
    ) -> Self {
        Self {
            auth,
            access_token,
            refresh_token: Some(refresh_token),
        }
    }
}

impl SessionAuth for SessionAuthService {
    async fn logout(&self) -> Result<(), ClientError> {
        match self.auth.api.logout(&self.access_token).await {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("Error logging out: {:?}", e);
                Err(ClientError::InternalError)
            }
        }
    }

    async fn list_users(&self) -> Result<Vec<User>, ClientError> {
        match self.auth.api.list_users(&self.access_token).await {
            Ok(users) => Ok(users.users),
            Err(ApiError::Request(_, ApiErrorCode::InvalidCredentials, _)) => {
                Err(ClientError::WrongCredentials)
            }
            Err(e) => {
                error!("Error listing users: {:?}", e);
                Err(ClientError::InternalError)
            }
        }
    }

    async fn refresh(&mut self) -> Result<Session, ClientError> {
        let refresh_token = match self.refresh_token {
            Some(ref refresh_token) => refresh_token,
            None => return Err(ClientError::MissingRefreshToken),
        };

        let session = match self.auth.api.refresh_access_token(refresh_token).await {
            Ok(session) => session,
            Err(e) => {
                error!("Error refreshing token: {:?}", e);
                return Err(ClientError::InternalError);
            }
        };

        self.access_token = session.access_token.clone().into();
        self.refresh_token = Some(session.refresh_token.clone());
        Ok(session)
    }
}
