use crate::helpers::{sign_up, spawn_test};
use axum::http::StatusCode;
use axum_supabase_auth::api::{Api, ApiError};
use matches::assert_matches;
use test_log::test;

#[test(tokio::test)]
async fn logout() {
    // Arrange
    let helpers = spawn_test();
    let client = helpers.autoconfirm_client;
    let (session, _) = sign_up(&client).await;

    // Act
    client
        .logout(&session.access_token)
        .await
        .expect("could not logout");

    // Token should be invalidated now
    let me = client.get_user(&session.access_token).await;

    // Assert
    assert_matches!(me, Err(ApiError::Request(StatusCode::FORBIDDEN, _, _)));
}
