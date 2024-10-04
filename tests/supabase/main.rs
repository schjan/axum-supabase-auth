use crate::helpers::spawn_test;

mod helpers;

#[tokio::test]
async fn test_supabase() {
    let _ = spawn_test().await;
}