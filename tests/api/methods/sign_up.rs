use crate::helpers::{generate_email, generate_password, spawn_test};
use axum::http::StatusCode;
use axum_supabase_auth::api::{Api, ApiError};
use axum_supabase_auth::{EmailOrPhone, User};
use matches::assert_matches;

#[tokio::test]
async fn sign_up() {
    // Arrange
    let helpers = spawn_test();
    let client = helpers.client;
    let email = generate_email();
    let password = generate_password();

    // Act
    let result = client
        .sign_up(EmailOrPhone::Email(email.clone()), password)
        .await
        .expect("Failed to sign up");

    // Assert
    let user = result.user().expect("Expected user but got session");
    assert_eq!(user.email, email);
}

#[tokio::test]
async fn sign_up_twice() {
    // Arrange
    let helpers = spawn_test();
    let client = helpers.client;
    let email = generate_email();
    let password = generate_password();
    let second_password = generate_password();

    // Act
    client
        .sign_up(EmailOrPhone::Email(email.clone()), password)
        .await
        .expect("First sign up failed");

    let result = client
        .sign_up(EmailOrPhone::Email(email.clone()), second_password)
        .await
        .expect("Second sign up failed, but should produce fake data");

    // Assert
    let user = result.user().expect("expected user but got session");
    assert_eq!(user.email, email);
}

#[tokio::test]
async fn sign_up_autoconfirm() {
    // Arrange
    let helpers = spawn_test();
    let client = helpers.autoconfirm_client;
    let email = generate_email();
    let password = generate_password();

    // Act
    let result = client
        .sign_up(EmailOrPhone::Email(email.clone()), password)
        .await
        .unwrap();

    // Assert
    let session = result.session().expect("expected session but got user");
    assert_eq!(session.user.email, email);
}

#[tokio::test]
async fn sign_up_autoconfirm_twice() {
    let helpers = spawn_test();
    let client = helpers.autoconfirm_client;
    let email = generate_email();
    let password = generate_password();
    let new_password = generate_password();

    client
        .sign_up(EmailOrPhone::Email(email.clone()), password)
        .await
        .expect("first sign up failed");

    let result = client
        .sign_up(EmailOrPhone::Email(email.clone()), new_password)
        .await;

    // Second sign up produces unprocessable entity as user can not directly be logged in.
    assert_matches!(
        result,
        Err(ApiError::Request(StatusCode::UNPROCESSABLE_ENTITY, _, _))
    );
}

#[tokio::test]
async fn sign_up_disabled() {
    let helpers = spawn_test();
    let client = helpers.signup_disabled_client;
    let email = generate_email();
    let password = generate_password();

    let result = client
        .sign_up(EmailOrPhone::Email(email.clone()), password)
        .await;

    // Should result in an HTTP Error
    assert_matches!(
        result,
        Err(ApiError::Request(StatusCode::UNPROCESSABLE_ENTITY, _, _))
    );
}

#[tokio::test]
async fn session_and_user_impl_as_ref() {
    let helpers = spawn_test();
    let client = helpers.client;
    let autoconfirm_client = helpers.autoconfirm_client;
    let email = generate_email();
    let second_email = generate_email();
    let password = generate_password();

    // Act
    let user_response = client
        .sign_up(EmailOrPhone::Email(email.clone()), &password)
        .await
        .expect("Failed to sign up");
    let session_response = autoconfirm_client
        .sign_up(EmailOrPhone::Email(second_email.clone()), password)
        .await
        .expect("Failed to sign up");

    // Assert
    let user: &User = user_response.as_ref();
    assert_eq!(user.email, email);
    let session: &User = session_response.as_ref();
    assert_eq!(session.email, second_email);
}
