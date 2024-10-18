use crate::helpers::{generate_password, sign_up, spawn_test};
use axum_supabase_auth::api::{Api, ApiError, OAuthErrorCode};
use matches::assert_matches;
use std::ops::Add;
use std::time::Duration;
use test_log::test;
use time::OffsetDateTime;

#[test(tokio::test)]
async fn refresh_token() {
    // Arrange
    let helpers = spawn_test();
    let client = helpers.autoconfirm_client;
    let (session, credentials) = sign_up(&client).await;

    // Act
    let result = client
        .refresh_access_token(session.refresh_token)
        .await
        .expect("refreshing access token failed");

    // Assert
    assert_eq!(result.user.email, credentials.email);
    assert!(!result.access_token.as_ref().is_empty());
    assert!(!result.refresh_token.as_ref().is_empty());
    assert!(result.expires_at > OffsetDateTime::now_utc().add(Duration::from_secs(60 * 30)));
    assert!(result.expires_at < OffsetDateTime::now_utc().add(Duration::from_secs(60 * 61)));
}

#[test(tokio::test)]
async fn refresh_token_wrong_token() {
    // Arrange
    let helpers = spawn_test();
    let client = helpers.autoconfirm_client;

    // Act
    let result = client.refresh_access_token(generate_password()).await;

    // Assert
    assert_matches!(
        result,
        Err(ApiError::OAuth(_, OAuthErrorCode::InvalidGrant, _))
    );
}
