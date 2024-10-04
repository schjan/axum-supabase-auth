use std::sync::LazyLock;
use tracing::subscriber::set_global_default;
use tracing_subscriber::{fmt, EnvFilter, Registry};
use tracing_subscriber::layer::SubscriberExt;

static TRACING: LazyLock<()> = LazyLock::new(|| {
    let subscriber = Registry::default()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "trace,hyper_util=warn".into()))
        .with(fmt::layer().with_writer(std::io::stdout));
    set_global_default(subscriber).expect("Failed to set subscriber");
});

const PROJECT_REFERENCE: &str = "project_ref";
const API_KEY: &str = "api_key";
const JWT_SECRET: &str = "secret";

pub async fn spawn_test() {
    LazyLock::force(&TRACING);
}
