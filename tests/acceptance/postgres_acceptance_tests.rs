use std::sync::Arc;

use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use cashback_rewards_rust::{
    adapter::{
        r#in::web::{self, WebState},
        out::persistence::{PgCashbackRepository, PgCategoryRepository, PgMerchantRepository},
    },
    application::{
        port::out::{CashbackRepository, CategoryRepository, MerchantRepository},
        service::{
            ListCustomerCashbackService, ManageProductCategoriesService, RecordPurchaseService,
            RegisterMerchantService, TotalProductCashbackService,
        },
    },
};
use serde_json::{Value, json};
use sqlx::{PgPool, postgres::PgPoolOptions};
use testcontainers::{
    GenericImage, ImageExt,
    core::{IntoContainerPort, WaitFor},
    runners::AsyncRunner,
};
use tokio::time::{Duration, Instant, sleep};
use tower::ServiceExt;

const POSTGRES_PORT: u16 = 5432;
const POSTGRES_DB: &str = "cashback";
const POSTGRES_USER: &str = "cashback";
const POSTGRES_PASSWORD: &str = "cashback";

async fn postgres_pool()
-> Result<(testcontainers::ContainerAsync<GenericImage>, PgPool), Box<dyn std::error::Error>> {
    let container = GenericImage::new("postgres", "18-alpine")
        .with_wait_for(WaitFor::message_on_stderr(
            "database system is ready to accept connections",
        ))
        .with_exposed_port(POSTGRES_PORT.tcp())
        .with_env_var("POSTGRES_DB", POSTGRES_DB)
        .with_env_var("POSTGRES_USER", POSTGRES_USER)
        .with_env_var("POSTGRES_PASSWORD", POSTGRES_PASSWORD)
        .start()
        .await?;

    let host = container.get_host().await?;
    let port = container.get_host_port_ipv4(POSTGRES_PORT.tcp()).await?;
    let database_url =
        format!("postgres://{POSTGRES_USER}:{POSTGRES_PASSWORD}@{host}:{port}/{POSTGRES_DB}");

    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        match PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await
        {
            Ok(pool) => return Ok((container, pool)),
            Err(_) if Instant::now() < deadline => {
                sleep(Duration::from_millis(250)).await;
            }
            Err(error) => return Err(error.into()),
        }
    }
}

fn build_app(pool: PgPool) -> axum::Router {
    let merchants: Arc<dyn MerchantRepository> = Arc::new(PgMerchantRepository::new(pool.clone()));
    let categories: Arc<dyn CategoryRepository> = Arc::new(PgCategoryRepository::new(pool.clone()));
    let cashbacks: Arc<dyn CashbackRepository> = Arc::new(PgCashbackRepository::new(pool));

    web::router(WebState::new(
        Arc::new(ListCustomerCashbackService::new(cashbacks.clone())),
        Arc::new(ManageProductCategoriesService::new(categories.clone())),
        Arc::new(RecordPurchaseService::new(
            merchants.clone(),
            categories,
            cashbacks.clone(),
        )),
        Arc::new(RegisterMerchantService::new(merchants)),
        Arc::new(TotalProductCashbackService::new(cashbacks)),
    ))
}

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

async fn reset(pool: &PgPool) -> Result<(), sqlx::Error> {
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
    let (_container, pool) = postgres_pool().await?;
    sqlx::migrate!("./migrations").run(&pool).await?;

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
