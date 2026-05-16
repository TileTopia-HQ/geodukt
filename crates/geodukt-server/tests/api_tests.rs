//! Tests for the REST API server.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use geodukt_server::create_router;
use tower::ServiceExt;

#[tokio::test]
async fn test_health_endpoint() {
    let app = create_router();
    let req = Request::builder()
        .uri("/health")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = axum::body::to_bytes(resp.into_body(), 1024).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["status"], "ok");
}

#[tokio::test]
async fn test_list_runs_empty() {
    let app = create_router();
    let req = Request::builder().uri("/runs").body(Body::empty()).unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = axum::body::to_bytes(resp.into_body(), 1024).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json, serde_json::json!([]));
}

#[tokio::test]
async fn test_get_run_not_found() {
    let app = create_router();
    let req = Request::builder()
        .uri("/runs/999")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_run_invalid_manifest() {
    let app = create_router();
    let req = Request::builder()
        .method("POST")
        .uri("/run")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"manifest": "not valid toml [["}"#))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}
