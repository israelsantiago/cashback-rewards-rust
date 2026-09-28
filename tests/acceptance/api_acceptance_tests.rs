use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use cashback_rewards_rust::bootstrap::build_app;
use serde_json::{Value, json};
use std::error::Error;
use tower::ServiceExt;

use crate::support::postgres::postgres_context;

async fn request(
    app: axum::Router,
    method: &str,
    uri: &str,
    body: Value,
) -> Result<axum::response::Response, Box<dyn Error>> {
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

async fn cashback_for(app: axum::Router, customer_id: &str) -> Result<Value, Box<dyn Error>> {
    let response = request(
        app,
        "GET",
        &format!("/api/customers/{customer_id}/cashback"),
        json!({}),
    )
    .await?;
    assert_eq!(response.status(), StatusCode::OK);
    Ok(serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX).await?,
    )?)
}

#[tokio::test]
async fn partner_purchase_is_visible_as_cashback() -> Result<(), Box<dyn Error>> {
    // This is a production-composition acceptance test: the HTTP router,
    // application services, outbound ports, PostgreSQL adapters and database
    // are all real. Only the PostgreSQL environment is test-managed.
    let context = postgres_context().await?;
    let app = build_app(context.pool.clone());

    assert_eq!(
        request(
            app.clone(),
            "POST",
            "/api/categories",
            json!({"mcc":"5411","name":"Groceries","cashbackRate":"0.02"}),
        )
        .await?
        .status(),
        StatusCode::CREATED
    );
    assert_eq!(
        request(
            app.clone(),
            "POST",
            "/api/merchants",
            json!({"name":"GreenGrocer","partner":true}),
        )
        .await?
        .status(),
        StatusCode::CREATED
    );
    assert_eq!(
        request(
            app.clone(),
            "POST",
            "/api/purchases",
            json!({
                "customerId":"cust-001",
                "merchantName":"GreenGrocer",
                "mcc":"5411",
                "amount":"80.00",
                "purchasedAt":"2026-05-01T10:00:00Z"
            }),
        )
        .await?
        .status(),
        StatusCode::CREATED
    );

    let body = cashback_for(app, "cust-001").await?;
    assert_eq!(body[0]["merchantName"], "GreenGrocer");
    assert_eq!(body[0]["productCategory"], "Groceries");
    assert_eq!(body[0]["cashbackAmount"].to_string(), "1.60");

    Ok(())
}

#[tokio::test]
async fn non_partner_and_below_threshold_purchases_create_no_record() -> Result<(), Box<dyn Error>>
{
    let context = postgres_context().await?;
    let app = build_app(context.pool.clone());

    request(
        app.clone(),
        "POST",
        "/api/categories",
        json!({"mcc":"5411","name":"Groceries","cashbackRate":"0.02"}),
    )
    .await?;
    request(
        app.clone(),
        "POST",
        "/api/merchants",
        json!({"name":"Corner Cafe","partner":false}),
    )
    .await?;
    request(
        app.clone(),
        "POST",
        "/api/merchants",
        json!({"name":"Tiny Grocer","partner":true}),
    )
    .await?;
    request(
        app.clone(),
        "POST",
        "/api/purchases",
        json!({
            "customerId":"cust-1",
            "merchantName":"Corner Cafe",
            "mcc":"5411",
            "amount":"80.00",
            "purchasedAt":"2026-05-01T10:00:00Z"
        }),
    )
    .await?;
    request(
        app.clone(),
        "POST",
        "/api/purchases",
        json!({
            "customerId":"cust-2",
            "merchantName":"Tiny Grocer",
            "mcc":"5411",
            "amount":"0.99",
            "purchasedAt":"2026-05-01T10:00:00Z"
        }),
    )
    .await?;

    assert_eq!(cashback_for(app.clone(), "cust-1").await?, json!([]));
    assert_eq!(cashback_for(app, "cust-2").await?, json!([]));

    Ok(())
}
