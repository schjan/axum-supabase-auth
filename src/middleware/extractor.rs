use super::{AuthState, Claims};
use crate::AuthTypes;
use axum::extract::{FromRef, FromRequestParts};
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{async_trait, Json};
use axum_extra::extract::CookieJar;
use serde_json::json;
use std::fmt::Debug;
use tracing::{trace, warn, Span};

pub type AuthClaims<T> =
    Claims<<T as AuthTypes>::AppData, <T as AuthTypes>::UserData, <T as AuthTypes>::AdditionalData>;

pub struct User<T: AuthTypes>(pub AuthClaims<T>);
pub struct MaybeUser<T: AuthTypes>(pub Option<AuthClaims<T>>);

pub const AUTH_COOKIE_NAME: &str = "portal-auth";
pub const REFRESH_COOKIE_NAME: &str = "portal-refresh";
pub const CSRF_VERIFIER_COOKIE_NAME: &str = "portal-token-code-verifier";

#[async_trait]
impl<S, T> FromRequestParts<S> for User<T>
where
    S: Send + Sync,
    T: AuthTypes,
    MaybeUser<T>: FromRequestParts<S, Rejection = AuthError>,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let user = MaybeUser::from_request_parts(parts, state).await?;
        if let Some(user) = user.0 {
            Ok(User(user))
        } else {
            Err(AuthError::MissingCredentials)
        }
    }
}

#[async_trait]
impl<S, T> FromRequestParts<S> for MaybeUser<T>
where
    S: Send + Sync,
    T: AuthTypes,
    AuthState<T>: FromRef<S>,
{
    type Rejection = AuthError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let jar = match CookieJar::from_request_parts(parts, state).await {
            Ok(jar) => jar,
            Err(err) => match err {},
        };

        let state = AuthState::<T>::from_ref(state);

        let token = jar.get(state.cookies().auth_cookie_name());
        let token = match token {
            Some(token) => token,
            None => {
                trace!("no auth cookie found");
                return Ok(MaybeUser(None));
            }
        };

        let claims = state.decode(token.value_trimmed()).map_err(|error| {
            warn!(error = ?error, "invalid token");
            AuthError::InvalidToken
        })?;

        trace!(claims = ?claims, "extracted user from cookie");
        Span::current().record("user_id", &claims.sub);

        Ok(MaybeUser(Some(claims)))
    }
}

// error types for axum errors
#[derive(Debug)]
pub enum AuthError {
    WrongCredentials,
    MissingCredentials,
    TokenCreation,
    InvalidToken,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AuthError::WrongCredentials => (StatusCode::UNAUTHORIZED, "Wrong credentials"),
            AuthError::MissingCredentials => (StatusCode::BAD_REQUEST, "Missing credentials"),
            AuthError::TokenCreation => (StatusCode::INTERNAL_SERVER_ERROR, "Token creation error"),
            AuthError::InvalidToken => (StatusCode::BAD_REQUEST, "Invalid token"),
        };
        let body = Json(json!({ "error": error_message }));
        (status, body).into_response()
    }
}
