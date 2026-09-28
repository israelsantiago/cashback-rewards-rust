use async_trait::async_trait;
use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode},
    routing::{get, post, put},
};
use cashback_rewards_rust::{
    adapter::r#in::web::{
        WebState, cashback_controller, category_controller, merchant_controller,
        product_cashback_controller, purchase_controller,
    },
    application::{
        ApplicationError,
        port::r#in::{
            ListCustomerCashbackUseCase, ManageProductCategoriesUseCase, RecordPurchaseUseCase,
            RegisterMerchantUseCase, TotalProductCashbackUseCase,
        },
    },
    domain::model::{CashbackRecord, Merchant, ProductCashbackTotal, ProductCategory},
};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};
use tower::ServiceExt;

#[derive(Default)]
struct ListUseCase {
    records: Mutex<Vec<CashbackRecord>>,
}
#[async_trait]
impl ListCustomerCashbackUseCase for ListUseCase {
    async fn list_for(&self, _: &str) -> Result<Vec<CashbackRecord>, ApplicationError> {
        Ok(self.records.lock().unwrap().clone())
    }
}
#[derive(Default)]
struct CategoryUseCase {
    category: Mutex<Option<ProductCategory>>,
    rate: Mutex<Option<Decimal>>,
}
#[async_trait]
impl ManageProductCategoriesUseCase for CategoryUseCase {
    async fn register(&self, c: ProductCategory) -> Result<(), ApplicationError> {
        *self.category.lock().unwrap() = Some(c);
        Ok(())
    }
    async fn configure_default_rate(&self, r: Decimal) -> Result<(), ApplicationError> {
        *self.rate.lock().unwrap() = Some(r);
        Ok(())
    }
}

type PurchaseValuesStore = Mutex<Option<(String, String, String, Decimal, DateTime<Utc>)>>;

type ControllerTestState = (
    WebState,
    Arc<ListUseCase>,
    Arc<CategoryUseCase>,
    Arc<PurchaseUseCase>,
    Arc<MerchantUseCase>,
    Arc<TotalUseCase>,
);

#[derive(Default)]
struct PurchaseUseCase {
    values: PurchaseValuesStore,
}
#[async_trait]
impl RecordPurchaseUseCase for PurchaseUseCase {
    async fn record(
        &self,
        c: &str,
        m: &str,
        mcc: &str,
        a: Decimal,
        t: DateTime<Utc>,
    ) -> Result<(), ApplicationError> {
        *self.values.lock().unwrap() = Some((c.into(), m.into(), mcc.into(), a, t));
        Ok(())
    }
}
#[derive(Default)]
struct MerchantUseCase {
    value: Mutex<Option<Merchant>>,
}
#[async_trait]
impl RegisterMerchantUseCase for MerchantUseCase {
    async fn register(&self, m: Merchant) -> Result<(), ApplicationError> {
        *self.value.lock().unwrap() = Some(m);
        Ok(())
    }
}
#[derive(Default)]
struct TotalUseCase {
    total: Mutex<Option<ProductCashbackTotal>>,
}
#[async_trait]
impl TotalProductCashbackUseCase for TotalUseCase {
    async fn total_for(&self, p: &str) -> Result<ProductCashbackTotal, ApplicationError> {
        Ok(self
            .total
            .lock()
            .unwrap()
            .clone()
            .unwrap_or_else(|| ProductCashbackTotal::new(p.into(), Decimal::ZERO, 0)))
    }
}

async fn request(app: Router, method: &str, uri: &str, body: Value) -> axum::response::Response {
    app.oneshot(
        Request::builder()
            .method(method)
            .uri(uri)
            .header("content-type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap(),
    )
    .await
    .expect("Axum router request should not fail")
}
fn state() -> ControllerTestState {
    let l = Arc::new(ListUseCase::default());
    let c = Arc::new(CategoryUseCase::default());
    let p = Arc::new(PurchaseUseCase::default());
    let m = Arc::new(MerchantUseCase::default());
    let t = Arc::new(TotalUseCase::default());
    (
        WebState::new(l.clone(), c.clone(), p.clone(), m.clone(), t.clone()),
        l,
        c,
        p,
        m,
        t,
    )
}

#[tokio::test]
async fn cashback_controller_returns_customer_cashback_as_json() {
    let (state, l, _, _, _, _) = state();
    *l.records.lock().unwrap() = vec![CashbackRecord {
        customer_id: "cust-001".into(),
        merchant_name: "GreenGrocer".into(),
        product_category: "Groceries".into(),
        cashback_amount: rust_decimal::dec!(2.40),
    }];
    let app = Router::new()
        .route(
            "/api/customers/{customer_id}/cashback",
            get(cashback_controller::list_customer_cashback),
        )
        .with_state(state);
    let r = request(app, "GET", "/api/customers/cust-001/cashback", json!({})).await;
    assert_eq!(r.status(), StatusCode::OK);
    let v: Value =
        serde_json::from_slice(&to_bytes(r.into_body(), usize::MAX).await.unwrap()).unwrap();
    let expected: Value = serde_json::from_str(
        r#"[{"merchantName":"GreenGrocer","productCategory":"Groceries","cashbackAmount":2.40}]"#,
    )
    .unwrap();
    assert_eq!(v, expected);
}

#[tokio::test]
async fn category_controller_registers_category_and_default_rate() {
    let (state, _, c, _, _, _) = state();
    let app = Router::new()
        .route(
            "/api/categories",
            post(category_controller::register_category),
        )
        .route(
            "/api/categories/default-rate",
            put(category_controller::configure_default_rate),
        )
        .with_state(state);
    let r = request(
        app.clone(),
        "POST",
        "/api/categories",
        json!({"mcc":"5411","name":"Groceries","cashbackRate":"0.02"}),
    )
    .await;
    assert_eq!(r.status(), StatusCode::CREATED);
    assert_eq!(
        *c.category.lock().unwrap(),
        Some(ProductCategory {
            mcc: "5411".into(),
            name: "Groceries".into(),
            cashback_rate: rust_decimal::dec!(0.02)
        })
    );
    let r = request(
        app,
        "PUT",
        "/api/categories/default-rate",
        json!({"cashbackRate":"0.005"}),
    )
    .await;
    assert_eq!(r.status(), StatusCode::NO_CONTENT);
    assert_eq!(*c.rate.lock().unwrap(), Some(rust_decimal::dec!(0.005)));
}

#[tokio::test]
async fn merchant_controller_registers_merchant_with_partner_flag() {
    let (state, _, _, _, m, _) = state();
    let app = Router::new()
        .route(
            "/api/merchants",
            post(merchant_controller::register_merchant),
        )
        .with_state(state);
    let r = request(
        app,
        "POST",
        "/api/merchants",
        json!({"name":"GreenGrocer","partner":true}),
    )
    .await;
    assert_eq!(r.status(), StatusCode::CREATED);
    assert_eq!(
        *m.value.lock().unwrap(),
        Some(Merchant {
            name: "GreenGrocer".into(),
            partner: true
        })
    );
}

#[tokio::test]
async fn product_cashback_controller_returns_product_total_as_json() {
    let (state, _, _, _, _, t) = state();
    *t.total.lock().unwrap() = Some(ProductCashbackTotal::new(
        "Groceries".into(),
        rust_decimal::dec!(4.00),
        2,
    ));
    let app = Router::new()
        .route(
            "/api/products/{product_category}/cashback-total",
            get(product_cashback_controller::get_product_total),
        )
        .with_state(state);
    let r = request(
        app,
        "GET",
        "/api/products/Groceries/cashback-total",
        json!({}),
    )
    .await;
    assert_eq!(r.status(), StatusCode::OK);
    let v: Value =
        serde_json::from_slice(&to_bytes(r.into_body(), usize::MAX).await.unwrap()).unwrap();
    let expected: Value =
        serde_json::from_str(r#"{"product":"Groceries","totalCashback":4.00,"recordCount":2}"#)
            .unwrap();
    assert_eq!(v, expected);
}

#[tokio::test]
async fn purchase_controller_passes_all_request_fields_to_use_case() {
    let (state, _, _, p, _, _) = state();
    let app = Router::new()
        .route("/api/purchases", post(purchase_controller::record_purchase))
        .with_state(state);
    let r=request(app,"POST","/api/purchases",json!({"customerId":"cust-001","merchantName":"GreenGrocer","mcc":"5411","amount":"80.00","purchasedAt":"2026-05-01T14:00:01Z"})).await;
    assert_eq!(r.status(), StatusCode::CREATED);
    let got = p.values.lock().unwrap().clone().unwrap();
    assert_eq!(got.0, "cust-001");
    assert_eq!(got.1, "GreenGrocer");
    assert_eq!(got.2, "5411");
    assert_eq!(got.3, rust_decimal::dec!(80.00));
    assert_eq!(
        got.4,
        "2026-05-01T14:00:01Z".parse::<DateTime<Utc>>().unwrap()
    );
}
