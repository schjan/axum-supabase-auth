use crate::{AppUser, MaybeAppUser, MyAuthTypes};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::Router;
use axum_supabase_auth::middleware::{AuthState, MaybeUser, User};

pub fn router() -> Router<AuthState<MyAuthTypes>> {
    Router::new()
        .route("/protected", post(protected))
        .route("/unprotected", get(unprotected))
}

async fn protected(User(claims): AppUser) -> impl IntoResponse {}

async fn unprotected(MaybeUser(claims): MaybeAppUser) -> impl IntoResponse {}
