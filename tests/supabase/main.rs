use crate::helpers::spawn_test;
use axum_supabase_auth::api::Api;

mod helpers;

#[tokio::test]
async fn health() {
    // Arrange
    let helpers = spawn_test().await;
    let client = helpers.api;

    // Act
    let result = client.health_check().await.expect("health check failed");

    // Assert
    assert_eq!(result.name, "GoTrue")
}
