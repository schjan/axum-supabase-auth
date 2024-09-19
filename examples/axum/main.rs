use crate::routes::AppState;
use axum::Router;
use axum_supabase_auth::middleware::{AuthState, MaybeUser, User};
use axum_supabase_auth::{AuthTypes, SupabaseAuth, SupabaseAuthConfig};
use clap::Parser;
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tracing::debug;

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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let auth = SupabaseAuthConfig::builder()
        .api_url(args.endpoint.parse()?)
        .jwt_secret(args.jwt_secret)
        .api_key(args.anon_key)
        .build();

    let auth = SupabaseAuth::new(auth)?;

    let state = AppState { auth: auth.state() };

    let app = Router::new().merge(auth.router()).with_state(state);

    let listener = TcpListener::bind("127.0.0.1:3000").await?;

    debug!("listening on {}", listener.local_addr()?);

    axum::serve(listener, app).await?;

    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppMetadata {
    pub groups: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserMetadata {}

#[derive(Debug, Serialize, Deserialize)]
pub struct Additional {
    pub user_role: Option<String>,
}

pub struct MyAuthTypes;

impl AuthTypes for MyAuthTypes {
    type AppData = AppMetadata;
    type UserData = UserMetadata;
    type AdditionalData = Additional;
}

pub type AppAuthState = AuthState<MyAuthTypes>;
pub type AppUser = User<MyAuthTypes>;
pub type MaybeAppUser = MaybeUser<MyAuthTypes>;
