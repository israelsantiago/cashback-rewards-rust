use std::{collections::HashMap, sync::{Arc, Mutex}};

use async_trait::async_trait;
use axum::{body::{to_bytes, Body}, http::{Request, StatusCode}};
use cashback_rewards_rust::{
    application::{port::{CashbackRepository, CategoryRepository, MerchantRepository, PortError}, service::{ListCustomerCashbackService, ManageProductCategoriesService, RecordPurchaseService, RegisterMerchantService, TotalProductCashbackService}, ApplicationState},
    domain::model::{CashbackRecord, Merchant, ProductCategory},
    web,
};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde_json::{json, Value};
use tower::ServiceExt;

#[derive(Default)]
struct MemoryMerchants {
    values: Mutex<HashMap<String, Merchant>>,
}
#[derive(Default)]
struct MemoryCategories {
    values: Mutex<HashMap<String, ProductCategory>>,
    default_rate: Mutex<Option<Decimal>>,
}
#[derive(Default)]
struct MemoryCashbacks {
    values: Mutex<Vec<CashbackRecord>>,
}

#[async_trait]
impl MerchantRepository for MemoryMerchants {
    async fn save(&self, merchant: &Merchant) -> Result<(), PortError> {
        self.values.lock().unwrap().insert(merchant.name.trim().to_lowercase(), merchant.clone());
        Ok(())
    }
    async fn find_by_name(&self, name: &str) -> Result<Option<Merchant>, PortError> {
        Ok(self.values.lock().unwrap().get(&name.trim().to_lowercase()).cloned())
    }
}

#[async_trait]
impl CategoryRepository for MemoryCategories {
    async fn save(&self, category: &ProductCategory) -> Result<(), PortError> {
        self.values.lock().unwrap().insert(category.mcc.clone(), category.clone());
        Ok(())
    }
    async fn save_default_rate(&self, rate: Decimal) -> Result<(), PortError> {
        *self.default_rate.lock().unwrap() = Some(rate);
        Ok(())
    }
    async fn default_rate(&self) -> Result<Option<Decimal>, PortError> {
        Ok(*self.default_rate.lock().unwrap())
    }
    async fn find_by_mcc(&self, mcc: &str) -> Result<Option<ProductCategory>, PortError> {
        Ok(self.values.lock().unwrap().get(mcc).cloned())
    }
}

#[async_trait]
impl CashbackRepository for MemoryCashbacks {
    async fn save(&self, record: &CashbackRecord) -> Result<(), PortError> {
        self.values.lock().unwrap().push(record.clone());
        Ok(())
    }
    async fn find_by_customer_id(&self, customer_id: &str) -> Result<Vec<CashbackRecord>, PortError> {
        Ok(self.values.lock().unwrap().iter().filter(|record| record.customer_id == customer_id).cloned().collect())
    }
    async fn total_for_product_category(&self, product_category: &str) -> Result<Decimal, PortError> {
        Ok(self.values.lock().unwrap().iter().filter(|record| record.product_category == product_category).map(|record| record.cashback_amount).sum())
    }
    async fn count_for_product_category(&self, product_category: &str) -> Result<i64, PortError> {
        Ok(self.values.lock().unwrap().iter().filter(|record| record.product_category == product_category).count() as i64)
    }
}

fn app() -> axum::Router {
    let merchants: Arc<dyn MerchantRepository> = Arc::new(MemoryMerchants::default());
    let categories: Arc<dyn CategoryRepository> = Arc::new(MemoryCategories::default());
    let cashbacks: Arc<dyn CashbackRepository> = Arc::new(MemoryCashbacks::default());

    web::router(ApplicationState::new(
        ListCustomerCashbackService::new(cashbacks.clone()),
        ManageProductCategoriesService::new(categories.clone()),
        RecordPurchaseService::new(merchants.clone(), categories, cashbacks.clone()),
        RegisterMerchantService::new(merchants),
        TotalProductCashbackService::new(cashbacks),
    ))
}

async fn request(app: axum::Router, method: &str, uri: &str, body: Value) -> axum::response::Response {
    app.oneshot(
        Request::builder()
            .method(method)
            .uri(uri)
            .header("content-type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap(),
    )
    .await
    .unwrap()
}

#[tokio::test]
async fn partner_purchase_is_visible_as_cashback() {
    let app = app();

    assert_eq!(request(app.clone(), "POST", "/api/categories", json!({"mcc":"5411","name":"Groceries","cashbackRate":"0.02"})).await.status(), StatusCode::CREATED);
    assert_eq!(request(app.clone(), "POST", "/api/merchants", json!({"name":"GreenGrocer","partner":true})).await.status(), StatusCode::CREATED);
    assert_eq!(request(app.clone(), "POST", "/api/purchases", json!({"customerId":"cust-001","merchantName":"GreenGrocer","mcc":"5411","amount":"80.00","purchasedAt":"2026-05-01T10:00:00Z"})).await.status(), StatusCode::CREATED);

    let response = request(app, "GET", "/api/customers/cust-001/cashback", json!({})).await;
    assert_eq!(response.status(), StatusCode::OK);
    let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let body: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(body[0]["merchantName"], "GreenGrocer");
    assert_eq!(body[0]["productCategory"], "Groceries");
    assert_eq!(body[0]["cashbackAmount"], "1.60");
}

#[tokio::test]
async fn non_partner_and_below_threshold_purchases_create_no_record() {
    let app = app();

    request(app.clone(), "POST", "/api/categories", json!({"mcc":"5411","name":"Groceries","cashbackRate":"0.02"})).await;
    request(app.clone(), "POST", "/api/merchants", json!({"name":"Corner Cafe","partner":false})).await;
    request(app.clone(), "POST", "/api/merchants", json!({"name":"Tiny Grocer","partner":true})).await;
    request(app.clone(), "POST", "/api/purchases", json!({"customerId":"cust-1","merchantName":"Corner Cafe","mcc":"5411","amount":"80.00","purchasedAt":"2026-05-01T10:00:00Z"})).await;
    request(app.clone(), "POST", "/api/purchases", json!({"customerId":"cust-2","merchantName":"Tiny Grocer","mcc":"5411","amount":"0.99","purchasedAt":"2026-05-01T10:00:00Z"})).await;

    let one = request(app.clone(), "GET", "/api/customers/cust-1/cashback", json!({})).await;
    let two = request(app, "GET", "/api/customers/cust-2/cashback", json!({})).await;
    assert_eq!(serde_json::from_slice::<Value>(&to_bytes(one.into_body(), usize::MAX).await.unwrap()).unwrap(), json!([]));
    assert_eq!(serde_json::from_slice::<Value>(&to_bytes(two.into_body(), usize::MAX).await.unwrap()).unwrap(), json!([]));
}
