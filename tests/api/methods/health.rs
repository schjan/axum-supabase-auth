use crate::helpers::spawn_test;

#[tokio::test]
async fn health() {
    // Arrange
    let helpers = spawn_test();
    let client = helpers.client;

    // Act
    let result = client.health_check().await.expect("health check failed");

    // Assert
    assert_eq!(result.name, "GoTrue")
}
