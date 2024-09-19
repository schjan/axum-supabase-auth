use crate::middleware::{AuthClaims, Decoder};
use crate::{AuthService, AuthTypes};
use axum::extract::FromRef;
use bon::Builder;
use std::sync::Arc;

#[derive(Builder, Clone)]
pub struct CookieConfig {
    auth_cookie_name: String,
    refresh_cookie_name: String,
    csrf_verifier_cookie_name: String,
}

impl CookieConfig {
    pub fn auth_cookie_name(&self) -> &str {
        &self.auth_cookie_name
    }

    pub fn refresh_cookie_name(&self) -> &str {
        &self.refresh_cookie_name
    }

    pub fn csrf_verifier_cookie_name(&self) -> &str {
        &self.csrf_verifier_cookie_name
    }
}

pub struct AuthState<T>
where
    T: AuthTypes,
{
    auth: AuthService,
    decoder: Arc<Decoder<T>>,
    cookies: CookieConfig,
}

impl<T> AuthState<T>
where
    T: AuthTypes,
{
    pub fn new(auth: AuthService, decoder: Arc<Decoder<T>>, cookies: CookieConfig) -> Self {
        Self {
            decoder,
            auth,
            cookies,
        }
    }

    pub fn auth(&self) -> &AuthService {
        &self.auth
    }

    pub fn decoder(&self) -> &Decoder<T> {
        &self.decoder
    }

    pub fn cookies(&self) -> &CookieConfig {
        &self.cookies
    }

    pub fn decode(&self, token: &str) -> Result<AuthClaims<T>, jsonwebtoken::errors::Error> {
        self.decoder.decode(token)
    }
}

impl<T> Clone for AuthState<T>
where
    T: AuthTypes,
{
    fn clone(&self) -> Self {
        Self {
            auth: self.auth.clone(),
            decoder: self.decoder.clone(),
            cookies: self.cookies.clone(),
        }
    }
}

impl<T> FromRef<AuthState<T>> for Arc<Decoder<T>>
where
    T: AuthTypes,
{
    fn from_ref(input: &AuthState<T>) -> Self {
        input.decoder.clone()
    }
}

impl<T> FromRef<AuthState<T>> for AuthService
where
    T: AuthTypes,
{
    fn from_ref(input: &AuthState<T>) -> Self {
        input.auth.clone()
    }
}
