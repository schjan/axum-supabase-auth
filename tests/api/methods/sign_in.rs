use crate::helpers::{generate_email, generate_password, sign_up, spawn_test};
use axum_supabase_auth::api::{Api, ApiError};
use axum_supabase_auth::EmailOrPhone;
use matches::assert_matches;
use reqwest::StatusCode;
use std::ops::Add;
use std::time::Duration;
use test_log::test;
use time::OffsetDateTime;

#[test(tokio::test)]
async fn sign_in() {
    // Arrange
    let helpers = spawn_test();
    let client = helpers.autoconfirm_client;
    let (_, credentials) = sign_up(&client).await;

    // Act
    let result = client
        .sign_in(
            EmailOrPhone::Email(credentials.email.clone()),
            credentials.password,
        )
        .await
        .expect("sign in failed");

    // Assert
    assert_eq!(result.user.email, credentials.email);
    assert!(!result.access_token.as_ref().is_empty());
    assert!(!result.refresh_token.as_ref().is_empty());
    assert!(result.expires_at > OffsetDateTime::now_utc().add(Duration::from_secs(60 * 30)));
    assert!(result.expires_at < OffsetDateTime::now_utc().add(Duration::from_secs(60 * 61)));
}

#[test(tokio::test)]
async fn sign_in_wrong_password() {
    // Arrange
    let helpers = spawn_test();
    let client = helpers.autoconfirm_client;
    let wrong_password = generate_password();
    let (_, credentials) = sign_up(&client).await;

    // Act
    let result = client
        .sign_in(
            EmailOrPhone::Email(credentials.email.clone()),
            wrong_password,
        )
        .await;

    // Assert
    assert_matches!(
        result,
        Err(ApiError::Request(StatusCode::BAD_REQUEST, _, _))
    );
}

#[test(tokio::test)]
async fn sign_in_non_existent() {
    // Arrange
    let helpers = spawn_test();
    let client = helpers.autoconfirm_client;
    let email = generate_email();
    let password = generate_password();

    // Act
    let result = client.sign_in(EmailOrPhone::Email(email), password).await;

    // Assert
    assert_matches!(
        result,
        Err(ApiError::Request(StatusCode::BAD_REQUEST, _, _))
    );
}
