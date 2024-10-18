use axum_supabase_auth::api::ApiClient;
use std::sync::LazyLock;
use std::time::Duration;
use tracing::subscriber::set_global_default;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{fmt, EnvFilter, Registry};

static TRACING: LazyLock<()> = LazyLock::new(|| {
    let subscriber = Registry::default()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "trace,hyper_util=warn".into()))
        .with(fmt::layer().with_writer(std::io::stdout));
    set_global_default(subscriber).expect("Failed to set subscriber");
});

const PROJECT_REFERENCE: &str = "project_ref";
const API_KEY: &str = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZS1kZW1vIiwicm9sZSI6ImFub24iLCJleHAiOjE5ODM4MTI5OTZ9.CRXP1A7WOeoJeXxjNni43kdQwgnWNReilDMblYTn_I0";
const JWT_SECRET: &str = "super-secret-jwt-token-with-at-least-32-characters-long";

pub struct TestApp {
    pub api: ApiClient,
}

pub async fn spawn_test() -> TestApp {
    LazyLock::force(&TRACING);

    let timeout = Duration::from_secs(1);
    let client = ApiClient::new(
        "http://localhost:54321/auth/v1/".try_into().unwrap(),
        timeout,
        API_KEY,
    );

    TestApp { api: client }
}
