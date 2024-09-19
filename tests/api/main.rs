use crate::helpers::spawn_test;
use anyhow::bail;
use axum_supabase_auth::api::SignUpError::UnableToSignUp;
use axum_supabase_auth::{EmailOrPhone, Session, User};
use fake::faker::internet::en::{FreeEmail, Password};
use fake::Fake;

mod helpers;

#[tokio::test]
async fn can_sign_up_autoconfirm() -> anyhow::Result<()> {
    let clients = spawn_test()?;
    let client = clients.autoconfirm_client;

    let email: String = FreeEmail().fake();
    let password: String = Password(8..20).fake();

    let result = client
        .sign_up(EmailOrPhone::Email(email.clone()), password)
        .await?;

    assert_eq!(AsRef::<User>::as_ref(&result).email, email);
    if let Some(session) = result.session() {
        assert_eq!(session.user.email, email);
    } else {
        bail!("expected session but got user");
    }

    Ok(())
}

#[tokio::test]
async fn sign_up_disabled() -> anyhow::Result<()> {
    let clients = spawn_test()?;
    let client = clients.signup_disabled_client;

    let email: String = FreeEmail().fake();
    let password: String = Password(8..20).fake();

    let result = client
        .sign_up(EmailOrPhone::Email(email.clone()), password)
        .await;
    assert!(matches!(result, Err(UnableToSignUp)));

    Ok(())
}

#[tokio::test]
async fn sign_up() -> anyhow::Result<()> {
    let clients = spawn_test()?;
    let client = clients.client;

    let email: String = FreeEmail().fake();
    let password: String = Password(8..20).fake();

    let result = client
        .sign_up(EmailOrPhone::Email(email.clone()), password)
        .await?;

    assert_eq!(AsRef::<User>::as_ref(&result).email, email);
    if let Some(user) = result.user() {
        assert_eq!(user.email, email);
    } else {
        bail!("expected session but got user");
    }

    Ok(())
}
