pub use cashback_rewards_rust::bootstrap::build_app;

use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use serde_json::{Value, json};
use tower::ServiceExt;

pub async fn request(
    app: axum::Router,
    method: &str,
    uri: &str,
    body: Value,
) -> Result<axum::response::Response, Box<dyn std::error::Error>> {
    Ok(app
        .oneshot(
            Request::builder()
                .method(method)
                .uri(uri)
                .header("content-type", "application/json")
                .body(Body::from(body.to_string()))?,
        )
        .await?)
}

pub async fn cashback_for(
    app: axum::Router,
    customer: &str,
) -> Result<Value, Box<dyn std::error::Error>> {
    let response = request(
        app,
        "GET",
        &format!("/api/customers/{customer}/cashback"),
        json!({}),
    )
    .await?;
    assert_eq!(response.status(), StatusCode::OK);
    Ok(serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX).await?,
    )?)
}
