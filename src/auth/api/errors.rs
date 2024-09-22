use axum::http::StatusCode;
use serde::Deserialize;
use std::fmt;
use std::fmt::Formatter;
use thiserror::Error;

/// https://github.com/supabase/auth/blob/master/internal/api/errors.go
#[derive(Debug, Error)]
pub enum ApiError {
    #[error("API request failed with status code {0} '{1}', message {2}")]
    Request(StatusCode, ApiErrorCode, String),
    #[error("OAuth request failed with status code {0}, error {1:?}, message {2}")]
    OAuth(StatusCode, OAuthErrorCode, String),
    #[error(transparent)]
    UnknownHTTP(#[from] reqwest::Error),
    #[error("URL parsing error: {0}")]
    UrlError(#[from] url::ParseError),
}

pub trait IntoApi {
    fn with_status(self, status_code: StatusCode) -> ApiError;
}

#[derive(Debug, Deserialize)]
pub struct ApiErrorResponse {
    pub code: u16,
    pub error_code: ApiErrorCode,
    pub msg: String,
}

impl IntoApi for ApiErrorResponse {
    fn with_status(self, status_code: StatusCode) -> ApiError {
        ApiError::Request(status_code, self.error_code, self.msg)
    }
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum ApiErrorCode {
    SignupDisabled,
    UserAlreadyExists,
    BadJwt,
    InvalidCredentials,
    #[serde(untagged)]
    Unknown(String),
}

impl fmt::Display for ApiErrorCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug, Deserialize)]
pub struct OAuthErrorResponse {
    pub error: OAuthErrorCode,
    #[serde(rename = "error_description")]
    pub description: Option<String>,
}

impl IntoApi for OAuthErrorResponse {
    fn with_status(self, status_code: StatusCode) -> ApiError {
        ApiError::OAuth(
            status_code,
            self.error,
            self.description.unwrap_or_default(),
        )
    }
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum OAuthErrorCode {
    InvalidGrant,
    #[serde(untagged)]
    Unknown(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_error_code_unknown_error() {
        let json = r#"{"error":"invalid_grant","error_description":"Invalid Refresh Token: Refresh Token Not Found"}"#;

        let result: OAuthErrorResponse = serde_json::from_str(json).unwrap();

        assert_eq!(result.error, OAuthErrorCode::InvalidGrant);
    }
}
