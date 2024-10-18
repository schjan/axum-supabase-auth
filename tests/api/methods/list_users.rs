use crate::helpers::{admin_token, sign_up, spawn_test};
use axum_supabase_auth::api::Api;
use test_log::test;

#[test(tokio::test)]
async fn list_users_admin() {
    // Arrange
    let helpers = spawn_test();
    let client = helpers.autoconfirm_client;
    let (session, _) = sign_up(&client).await;
    let user = session.user;

    let token = admin_token();

    // Act
    let result = client.list_users(token).await.unwrap();

    // Assert
    assert!(!result.users.is_empty());
    assert!(result.users.iter().any(|u| u.email == user.email));
}
