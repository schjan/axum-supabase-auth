mod auth;
mod handlers;
pub mod middleware;

use crate::handlers::auth_router;
use crate::middleware::{AuthState, CookieConfig, Decoder};
pub use auth::api;
pub use auth::service::*;
pub use auth::types::*;
pub use auth::{Auth, SessionAuth};
use axum::extract::FromRef;
use axum::Router;
use bon::Builder;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::sync::Arc;
use thiserror::Error;

pub trait AuthTypes {
    type AppData: Serialize + for<'de> Deserialize<'de> + Send + Debug;
    type UserData: Serialize + for<'de> Deserialize<'de> + Send + Debug;
    type AdditionalData: Serialize + for<'de> Deserialize<'de> + Send + Debug;
}

#[derive(Builder)]
#[builder()]
pub struct SupabaseAuthConfig {
    pub jwt_secret: String,
    pub api_url: Url,
    pub api_key: String,

    #[builder(into, default = "sb-auth")]
    pub auth_cookie_name: String,
    #[builder(into, default = "sb-refresh")]
    pub refresh_cookie_name: String,
    #[builder(into, default = "sb-token-verifier")]
    pub csrf_verifier_cookie_name: String,
}

#[derive(Clone)]
pub struct SupabaseAuth<T>
where
    T: AuthTypes + Send + Sync + 'static,
{
    state: AuthState<T>,
}

impl<T> SupabaseAuth<T>
where
    T: AuthTypes + Send + Sync + 'static,
{
    pub fn new(conf: SupabaseAuthConfig) -> Result<Self, SupabaseAuthError> {
        let service = AuthService::new(conf.api_url, &conf.api_key);

        let decoder = Arc::new(Decoder::new(&conf.jwt_secret));

        let cookies = CookieConfig::builder()
            .auth_cookie_name(conf.auth_cookie_name)
            .csrf_verifier_cookie_name(conf.csrf_verifier_cookie_name)
            .refresh_cookie_name(conf.refresh_cookie_name)
            .build();

        let state = AuthState::new(service, decoder, cookies);

        Ok(Self { state })
    }

    pub fn router<S>(&self) -> Router<S>
    where
        S: Clone + Send + Sync + 'static,
        AuthState<T>: FromRef<S>,
    {
        auth_router()
    }

    pub fn state(&self) -> AuthState<T> {
        self.state.clone()
    }
}

#[derive(Error, Debug)]
pub enum SupabaseAuthError {}
