use crate::middleware::{AuthState, CookieConfig};
use crate::{AuthTypes, DefaultAuthTypes, Session};
use axum::extract::FromRef;
use axum::routing::{get, post};
use axum::Router;
use axum_extra::extract::cookie::{Cookie, SameSite};
use axum_extra::extract::CookieJar;
use std::ops::Add;
use time::{Duration, OffsetDateTime};

pub fn auth_router<T, S>() -> Router<S>
where
    T: AuthTypes + Send + Sync + 'static,
    S: Clone + Send + Sync + 'static,
    AuthState<T>: FromRef<S>,
{
    Router::new()
        .route("/login", post(post::login))
        .route("/logout", post(post::logout))
        .route("/login/github", get(get::login_github))
        .route("/login/confirm", get(get::login_confirm))
}

mod post {
    use crate::auth::SessionAuth;
    use crate::handlers::set_cookies_from_session;
    use crate::middleware::{AccessToken, MaybeUser, SomeAccessToken};
    use crate::{Auth, AuthTypes, EmailOrPhone};
    use crate::{AuthState, DefaultAuthTypes};
    use axum::extract::State;
    use axum::http::StatusCode;
    use axum::response::{IntoResponse, Redirect};
    use axum::{debug_handler, Form};
    use axum_extra::extract::CookieJar;
    use serde::Deserialize;
    use tracing::warn;

    #[derive(Debug, Clone, Deserialize)]
    pub struct Credentials {
        pub email: String,
        pub password: String,
        pub next: Option<String>,
    }

    pub async fn login<T>(
        jar: CookieJar,
        State(auth): State<AuthState<T>>,
        MaybeUser(claims): MaybeUser<T>,
        Form(creds): Form<Credentials>,
    ) -> impl IntoResponse
    where
        T: AuthTypes,
    {
        if claims.is_some() {
            return Redirect::to("/").into_response();
        }

        let session = match auth
            .auth()
            .sign_in(EmailOrPhone::Email(creds.email), &creds.password)
            .await
        {
            Ok(session) => session,
            // TODO: handle diffferent errors.
            Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        };

        let jar = set_cookies_from_session(auth.cookies(), jar, session);
        let next = creds.next.unwrap_or_else(|| "/profile".to_string());

        (jar, Redirect::to(&next)).into_response()
    }

    pub async fn logout<T>(
        jar: CookieJar,
        State(state): State<AuthState<T>>,
        token: AccessToken<T>,
    ) -> impl IntoResponse
    where
        T: AuthTypes,
    {
        let jar = jar.remove(state.cookies().refresh_cookie_name().to_string());
        let jar = jar.remove(state.cookies().auth_cookie_name().to_string());

        let client = state.auth().with_token(token.into());
        if let Err(err) = client.logout().await {
            warn!(%err, "logout failed");
            return (jar, StatusCode::INTERNAL_SERVER_ERROR).into_response();
        };

        (jar, Redirect::to("/login")).into_response()
    }
}

mod get {
    use crate::handlers::set_cookies_from_session;
    use crate::middleware::AuthState;
    use crate::{Auth, AuthTypes, OAuthRequest};
    use axum::extract::{Query, State};
    use axum::http::StatusCode;
    use axum::response::{IntoResponse, Redirect};
    use axum_extra::extract::cookie::Cookie;
    use axum_extra::extract::CookieJar;
    use serde::Deserialize;
    use std::ops::Add;
    use time::{Duration, OffsetDateTime};

    pub async fn login_github<T>(
        jar: CookieJar,
        State(state): State<AuthState<T>>,
    ) -> impl IntoResponse
    where
        T: AuthTypes,
    {
        let response = match state.auth().create_oauth_url(OAuthRequest {
            provider: "github".to_string(),
            redirect_to: "https%3A%2F%2Fhp-rs-htmx.fly.dev%2Flogin%2Fconfirm".to_string(),
        }) {
            Ok(response) => response,
            Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        };

        let csrf_token = Cookie::build((
            state.cookies().csrf_verifier_cookie_name().to_string(),
            response.csrf_token,
        ))
        .expires(OffsetDateTime::now_utc().add(Duration::minutes(2)))
        .secure(true);

        let jar = jar.add(csrf_token.build());

        (jar, Redirect::to(&response.supabase_url)).into_response()
    }

    #[derive(Deserialize)]
    pub struct ConfirmParams {
        code: String,
    }

    pub async fn login_confirm<T>(
        jar: CookieJar,
        State(state): State<AuthState<T>>,
        Query(ConfirmParams { code }): Query<ConfirmParams>,
    ) -> impl IntoResponse
    where
        T: AuthTypes,
    {
        let csrf_token = match jar.get(state.cookies().csrf_verifier_cookie_name()) {
            Some(csrf_token) => csrf_token,
            None => return StatusCode::UNAUTHORIZED.into_response(),
        };

        let session = match state
            .auth()
            .exchange_code_for_session(&code, csrf_token.value_trimmed())
            .await
        {
            Ok(session) => session,
            Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        };

        let jar = set_cookies_from_session(state.cookies(), jar, session);
        let jar = jar.remove(state.cookies().csrf_verifier_cookie_name().to_string());

        (jar, Redirect::to("/profile")).into_response()
    }
}

fn set_cookies_from_session(
    cookie_config: &CookieConfig,
    jar: CookieJar,
    session: Session,
) -> CookieJar {
    let expires = OffsetDateTime::now_utc().add(Duration::seconds(session.expires_in as i64));
    let auth_cookie = Cookie::build((
        cookie_config.auth_cookie_name().to_string(),
        session.access_token,
    ))
    .path("/")
    .secure(true)
    .expires(expires)
    .http_only(false)
    .same_site(SameSite::Lax)
    .build();

    let refresh_cookie = Cookie::build((
        cookie_config.refresh_cookie_name().to_string(),
        session.refresh_token,
    ))
    .path("/")
    .secure(true)
    .expires(expires)
    .http_only(false)
    .same_site(SameSite::Lax)
    .build();

    jar.add(auth_cookie).add(refresh_cookie)
}
