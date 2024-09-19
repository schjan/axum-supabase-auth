use crate::helpers::spawn_test;
use anyhow::bail;
use axum_supabase_auth::api::ApiError;
use axum_supabase_auth::{EmailOrPhone, Session, User};
use fake::faker::internet::en::{FreeEmail, Password};
use fake::Fake;
use reqwest::StatusCode;

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
    match result {
        Err(ApiError::HttpError(_, StatusCode::UNPROCESSABLE_ENTITY)) => {}
        _ => bail!("expected HTTP Error 422, but got: {:?}", result),
    }

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