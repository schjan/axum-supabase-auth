use crate::{AppState, AppUser, MaybeAppUser};
use askama_axum::IntoResponse;
use axum::extract::Query;
use axum::response::Redirect;
use axum::routing::get;
use axum::Router;
use axum_supabase_auth::middleware::{MaybeUser, User};
use serde::Deserialize;

mod templates;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(index))
        .route("/login", get(login))
        .route("/register", get(register))
        .route("/profile", get(profile))
}

async fn index(MaybeUser(_claims): MaybeAppUser) -> impl IntoResponse {
    templates::Index
}

// This allows us to extract the "next" field from the query string. We use this
// to redirect after log in.
#[derive(Debug, Deserialize)]
pub struct NextUrl {
    next: Option<String>,
}

async fn login(MaybeUser(user): MaybeAppUser,
               Query(NextUrl { next }): Query<NextUrl>) -> impl IntoResponse {
    if user.is_some() {
        return Redirect::to("/").into_response();
    }
    
    templates::Login {
        next: next.as_deref(),
    }.into_response()
}

async fn register(MaybeUser(user): MaybeAppUser) -> impl IntoResponse {
    if user.is_some() {
        return Redirect::to("/").into_response();
    }

    templates::Register.into_response()
}

async fn profile(User(claims): AppUser) -> impl IntoResponse {}
