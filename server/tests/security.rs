use axum::body::Body;
use axum::http::{Request, StatusCode};
use content_planner_server::{app, new_state};
use http_body_util::BodyExt;
use tower::ServiceExt;

async fn post_json(
    app: axum::Router,
    uri: &str,
    ip: &str,
    body: serde_json::Value,
) -> axum::response::Response {
    app.oneshot(
        Request::builder()
            .method("POST")
            .uri(uri)
            .header("content-type", "application/json")
            .header("x-forwarded-for", ip)
            .body(Body::from(body.to_string()))
            .unwrap(),
    )
    .await
    .unwrap()
}

async fn body_json(res: axum::response::Response) -> serde_json::Value {
    let bytes = res.into_body().collect().await.unwrap().to_bytes();
    serde_json::from_slice(&bytes).unwrap()
}

#[tokio::test]
async fn cors_does_not_reflect_arbitrary_origins() {
    let app = app(new_state());
    let res = app
        .oneshot(
            Request::builder()
                .method("OPTIONS")
                .uri("/api/health")
                .header("origin", "https://evil.example")
                .header("access-control-request-method", "GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(
        res.headers().get("access-control-allow-origin").is_none(),
        "an arbitrary origin must never be reflected with credentials"
    );
}

#[tokio::test]
async fn auth_endpoints_are_rate_limited_per_ip() {
    let app = app(new_state());
    let body = serde_json::json!({
        "oldPassword": "not-the-password",
        "newPassword": "longenough123"
    });

    // The limiter runs before authentication, so these are all 401 until the
    // per-IP budget is exhausted.
    for _ in 0..20 {
        let res = post_json(
            app.clone(),
            "/api/auth/change-password",
            "203.0.113.10",
            body.clone(),
        )
        .await;
        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);
    }

    let limited = post_json(
        app.clone(),
        "/api/auth/change-password",
        "203.0.113.10",
        body.clone(),
    )
    .await;
    assert_eq!(limited.status(), StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(body_json(limited).await["error_code"], "TOO_MANY_REQUESTS");

    // Another client IP keeps its own bucket.
    let other = post_json(app, "/api/auth/change-password", "203.0.113.11", body).await;
    assert_eq!(other.status(), StatusCode::UNAUTHORIZED);
}
