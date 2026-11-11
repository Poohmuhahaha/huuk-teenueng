//! Live R2 round-trip test. Skipped unless the R2_* environment is configured.

use content_planner_server::storage::R2Store;

#[tokio::test]
async fn r2_put_get_head_delete_round_trip() {
    let Some(r2) = R2Store::from_env() else {
        eprintln!("skipping R2 test: R2_* environment is not configured");
        return;
    };

    let key = format!(
        "tests/{}-{}.txt",
        std::process::id(),
        chrono::Utc::now().timestamp_nanos_opt().unwrap_or_default()
    );
    let body = b"huuk r2 smoke test".to_vec();

    r2.put(&key, body.clone(), "text/plain")
        .await
        .expect("R2 put");
    assert_eq!(r2.get(&key).await.expect("R2 get"), Some(body));
    assert!(r2.head(&key).await.expect("R2 head"));
    r2.delete(&key).await.expect("R2 delete");
    assert_eq!(r2.get(&key).await.expect("R2 get after delete"), None);
}
