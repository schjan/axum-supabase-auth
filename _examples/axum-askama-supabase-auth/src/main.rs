use crate::routes::router;
use anyhow::Context;
use axum::extract::FromRef;
use axum_supabase_auth::middleware::{AuthState, Empty, MaybeUser, User};
use axum_supabase_auth::{AuthTypes, SupabaseAuth, SupabaseAuthConfig};
use clap::Parser;
use serde::{Deserialize, Serialize};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod routes;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    #[arg(short, long)]
    endpoint: String,
    #[arg(short, long)]
    jwt_secret: String,
    #[arg(short, long)]
    anon_key: String,
}

pub struct MyAuthTypes;

impl AuthTypes for MyAuthTypes {
    type AppData = Empty;
    type UserData = Empty;
    type AdditionalData = Additional;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Additional {
    pub user_role: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Setup tracing.
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("{}=debug", env!("CARGO_CRATE_NAME")).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let auth = SupabaseAuthConfig::builder()
        .api_url(args.endpoint.parse()?)
        .jwt_secret(args.jwt_secret)
        .api_key(args.anon_key)
        .build();

    let auth = SupabaseAuth::new(auth)?;

    let state = AppState { auth: auth.state() };

    // See routes/mod.rs for Router definition.
    let app = router().merge(auth.router()).with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .context("failed to bind TcpListener")?;

    tracing::debug!(
        "listening on {}",
        listener
            .local_addr()
            .context("failed to return local address")?
    );

    axum::serve(listener, app).await?;

    Ok(())
}

#[derive(FromRef, Clone)]
struct AppState {
    pub auth: AppAuthState,
}

pub type AppAuthState = AuthState<MyAuthTypes>;
pub type AppUser = User<MyAuthTypes>;
pub type MaybeAppUser = MaybeUser<MyAuthTypes>;
