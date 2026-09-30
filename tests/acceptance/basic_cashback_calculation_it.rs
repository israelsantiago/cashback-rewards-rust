use crate::support::{
    acceptance::AcceptanceFixture, postgres::postgres_context, runtime::run_async,
};
use rust_decimal::dec;
#[test]
fn partner_purchase_is_visible_as_cashback() -> Result<(), Box<dyn std::error::Error>> {
    run_async(async {
        let pool = postgres_context().await?;
        let fixture = AcceptanceFixture::new(pool, "partner-cashback");
        let _timer = fixture.timer("partner_purchase_is_visible_as_cashback");

        let category = fixture.register_category("Groceries", dec!(0.02)).await?;
        let merchant = fixture.register_merchant("GreenGrocer", true).await?;
        fixture
            .record_purchase("cust-001", &merchant, &category, dec!(80.00))
            .await?;

        let body = fixture.cashback_for("cust-001").await?;
        assert_eq!(body.as_array().map(Vec::len), Some(1));
        assert_eq!(body[0]["merchantName"], merchant.merchant.name);
        assert_eq!(body[0]["productCategory"], category.category.name);
        assert_eq!(body[0]["cashbackAmount"].to_string(), "1.60");

        Ok::<(), Box<dyn std::error::Error>>(())
    })
}

#[test]
fn non_partner_and_below_threshold_purchases_create_no_record()
-> Result<(), Box<dyn std::error::Error>> {
    run_async(async {
        let pool = postgres_context().await?;
        let fixture = AcceptanceFixture::new(pool, "no-cashback");
        let _timer = fixture.timer("non_partner_and_below_threshold_purchases_create_no_record");

        let category = fixture.register_category("Groceries", dec!(0.02)).await?;
        let non_partner = fixture.register_merchant("CornerCafe", false).await?;
        let partner = fixture.register_merchant("TinyGrocer", true).await?;

        fixture
            .record_purchase("cust-1", &non_partner, &category, dec!(80.00))
            .await?;
        fixture
            .record_purchase("cust-2", &partner, &category, dec!(0.99))
            .await?;

        assert_eq!(fixture.cashback_for("cust-1").await?, serde_json::json!([]));
        assert_eq!(fixture.cashback_for("cust-2").await?, serde_json::json!([]));

        Ok::<(), Box<dyn std::error::Error>>(())
    })
}
