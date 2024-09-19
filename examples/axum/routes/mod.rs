mod auth;
mod business;

use crate::AppAuthState;
use axum::extract::FromRef;

#[derive(FromRef, Clone)]
pub struct AppState {
    pub auth: AppAuthState,
}
