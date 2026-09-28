use crate::support::postgres::postgres_context;
use cashback_rewards_rust::bootstrap::build_app;

use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use serde_json::{Value, json};
use tower::ServiceExt;

async fn request(
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

async fn reset(pool: &sqlx::PgPool) -> Result<(), sqlx::Error> {
    sqlx::query("TRUNCATE TABLE cashback_record, product_category, default_cashback_rate, merchant RESTART IDENTITY")
        .execute(pool)
        .await?;
    Ok(())
}

async fn cashback_for(
    app: axum::Router,
    customer_id: &str,
) -> Result<Value, Box<dyn std::error::Error>> {
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
async fn postgres_acceptance_covers_cashback_scenarios() -> Result<(), Box<dyn std::error::Error>> {
    let context = postgres_context().await?;
    let pool = context.pool;

    let merchant_table_exists: bool =
        sqlx::query_scalar("SELECT to_regclass('public.merchant') IS NOT NULL")
            .fetch_one(&pool)
            .await?;
    let cashback_table_exists: bool =
        sqlx::query_scalar("SELECT to_regclass('public.cashback_record') IS NOT NULL")
            .fetch_one(&pool)
            .await?;
    assert!(merchant_table_exists);
    assert!(cashback_table_exists);

    let router = build_app(pool.clone());

    // Java: BasicCashbackCalculationIT — partner merchant earns cashback.
    request(
        router.clone(),
        "POST",
        "/api/categories",
        json!({"mcc":"5411","name":"Groceries","cashbackRate":"0.02"}),
    )
    .await?;
    request(
        router.clone(),
        "POST",
        "/api/merchants",
        json!({"name":"GreenGrocer","partner":true}),
    )
    .await?;
    request(
        router.clone(),
        "POST",
        "/api/purchases",
        json!({"customerId":"cust-001","merchantName":"GreenGrocer","mcc":"5411","amount":"80.00","purchasedAt":"2026-05-01T10:00:00Z"}),
    )
    .await?;
    let records = cashback_for(router.clone(), "cust-001").await?;
    assert_eq!(records.as_array().unwrap().len(), 1);
    assert_eq!(records[0]["cashbackAmount"].to_string(), "1.60");

    reset(&pool).await?;

    // Java: BasicCashbackCalculationIT — non-partner merchant earns nothing.
    let router = build_app(pool.clone());
    request(
        router.clone(),
        "POST",
        "/api/categories",
        json!({"mcc":"5411","name":"Groceries","cashbackRate":"0.02"}),
    )
    .await?;
    request(
        router.clone(),
        "POST",
        "/api/merchants",
        json!({"name":"Corner Cafe","partner":false}),
    )
    .await?;
    request(
        router.clone(),
        "POST",
        "/api/purchases",
        json!({"customerId":"cust-002","merchantName":"Corner Cafe","mcc":"5411","amount":"80.00","purchasedAt":"2026-05-01T10:00:00Z"}),
    )
    .await?;
    assert_eq!(cashback_for(router.clone(), "cust-002").await?, json!([]));

    reset(&pool).await?;

    // Java: MerchantCategoriesAndEligibilityIT — unmapped MCC uses default rate.
    let router = build_app(pool.clone());
    request(
        router.clone(),
        "PUT",
        "/api/categories/default-rate",
        json!({"cashbackRate":"0.005"}),
    )
    .await?;
    request(
        router.clone(),
        "POST",
        "/api/merchants",
        json!({"name":"City Pharmacy","partner":true}),
    )
    .await?;
    request(
        router.clone(),
        "POST",
        "/api/purchases",
        json!({"customerId":"cust-default","merchantName":"City Pharmacy","mcc":"5912","amount":"100.00","purchasedAt":"2026-05-01T10:00:00Z"}),
    )
    .await?;
    let records = cashback_for(router.clone(), "cust-default").await?;
    assert_eq!(records[0]["productCategory"], "Other");
    assert_eq!(records[0]["cashbackAmount"].to_string(), "0.50");

    reset(&pool).await?;

    // Java: MinimumPurchaseThresholdIT — amount below $1 earns nothing.
    let router = build_app(pool.clone());
    request(
        router.clone(),
        "POST",
        "/api/categories",
        json!({"mcc":"5411","name":"Groceries","cashbackRate":"0.02"}),
    )
    .await?;
    request(
        router.clone(),
        "POST",
        "/api/merchants",
        json!({"name":"Tiny Grocer","partner":true}),
    )
    .await?;
    request(
        router.clone(),
        "POST",
        "/api/purchases",
        json!({"customerId":"cust-threshold","merchantName":"Tiny Grocer","mcc":"5411","amount":"0.99","purchasedAt":"2026-05-01T10:00:00Z"}),
    )
    .await?;
    assert_eq!(
        cashback_for(router.clone(), "cust-threshold").await?,
        json!([])
    );

    reset(&pool).await?;

    // Java: TotalCashbackPerProductIT — aggregate across customers.
    let router = build_app(pool.clone());
    request(
        router.clone(),
        "POST",
        "/api/categories",
        json!({"mcc":"5411","name":"Groceries","cashbackRate":"0.02"}),
    )
    .await?;
    request(
        router.clone(),
        "POST",
        "/api/merchants",
        json!({"name":"Market-A","partner":true}),
    )
    .await?;
    request(
        router.clone(),
        "POST",
        "/api/merchants",
        json!({"name":"Market-B","partner":true}),
    )
    .await?;
    request(
        router.clone(),
        "POST",
        "/api/purchases",
        json!({"customerId":"cust-001","merchantName":"Market-A","mcc":"5411","amount":"120.00","purchasedAt":"2026-05-01T10:00:00Z"}),
    )
    .await?;
    request(
        router.clone(),
        "POST",
        "/api/purchases",
        json!({"customerId":"cust-002","merchantName":"Market-B","mcc":"5411","amount":"80.00","purchasedAt":"2026-05-01T10:00:00Z"}),
    )
    .await?;
    let response = request(
        router,
        "GET",
        "/api/products/Groceries/cashback-total",
        json!({}),
    )
    .await?;
    assert_eq!(response.status(), StatusCode::OK);
    let total: Value = serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await?)?;
    assert_eq!(total["totalCashback"].to_string(), "4.00");
    assert_eq!(total["recordCount"], 2);

    Ok(())
}
