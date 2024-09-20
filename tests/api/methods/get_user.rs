use crate::helpers::{generate_password, sign_up, spawn_test};
use axum::http::StatusCode;
use axum_supabase_auth::api::ApiError;
use matches::assert_matches;

#[tokio::test]
async fn get_user() {
    // Arrange
    let helpers = spawn_test();
    let client = helpers.autoconfirm_client;
    let (session, credentials) = sign_up(&client).await;

    // Act
    let me = client
        .get_user(&session.access_token)
        .await
        .expect("could not get user");

    // Assert
    assert_eq!(me.email, credentials.email);
}

#[tokio::test]
async fn get_user_wrong_token() {
    // Arrange
    let helpers = spawn_test();
    let client = helpers.autoconfirm_client;

    // Act
    let result = client.get_user(generate_password()).await;

    // Assert
    assert_matches!(result, Err(ApiError::HttpError(_, StatusCode::FORBIDDEN)));
}
