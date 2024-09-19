use crate::MyAuthTypes;
use axum::routing::get;
use axum::Router;
use axum_supabase_auth::middleware::AuthState;
use serde::Deserialize;

pub struct LoginTemplate {
    next: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Credentials {
    pub email: String,
    pub password: String,
    pub next: Option<String>,
}

// This allows us to extract the "next" field from the query string. We use this
// to redirect after log in.
#[derive(Debug, Deserialize)]
pub struct NextUrl {
    next: Option<String>,
}

pub fn router() -> Router<AuthState<MyAuthTypes>> {
    Router::new().route("/login", get(get::login))
}

mod get {
    use crate::routes::auth::NextUrl;
    use axum::extract::Query;
    use axum::response::IntoResponse;

    pub async fn login(Query(NextUrl { next }): Query<NextUrl>) -> impl IntoResponse {
        "ok".into_response()
        // LoginTemplate { next }
    }
}
