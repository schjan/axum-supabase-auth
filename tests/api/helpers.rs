use axum_supabase_auth::api::Api;
use axum_supabase_auth::{EmailOrPhone, Session, User};
use fake::faker::internet::en::{FreeEmail, Password};
use fake::Fake;
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
const API_KEY: &str = "api_key";
const JWT_SECRET: &str = "secret";

pub struct Clients {
    pub client: Api,
    pub autoconfirm_client: Api,
    pub signup_disabled_client: Api,
}

pub fn spawn_test() -> Clients {
    LazyLock::force(&TRACING);

    let timeout = Duration::from_secs(1);
    let client = Api::new(
        "http://localhost:9999".try_into().unwrap(),
        timeout,
        API_KEY,
    );
    let autoconfirm_client = Api::new(
        "http://localhost:9998".try_into().unwrap(),
        timeout,
        API_KEY,
    );
    let signup_disabled_client = Api::new(
        "http://localhost:9997".try_into().unwrap(),
        timeout,
        API_KEY,
    );

    Clients {
        client,
        autoconfirm_client,
        signup_disabled_client,
    }
}

pub struct Credentials {
    pub email: String,
    pub password: String,
}

pub async fn sign_up(client: &Api) -> (Session, Credentials) {
    let email = generate_email();
    let password = generate_password();

    let result = client
        .sign_up(EmailOrPhone::Email(email.clone()), &password)
        .await
        .expect("Failed to sign up");

    let credentials = Credentials { email, password };

    let session = result.session().expect("expected session");

    (session, credentials)
}

pub fn generate_email() -> String {
    FreeEmail().fake()
}

pub fn generate_password() -> String {
    Password(8..20).fake()
}
