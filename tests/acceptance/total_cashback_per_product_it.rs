use rust_decimal::dec;
use crate::support::{
    acceptance::AcceptanceFixture, postgres::postgres_context, runtime::run_async,
};
#[test]
fn total_cashback_for_product_sums_records_across_customers() -> Result<(), Box<dyn std::error::Error>> {
    run_async(async {
        let pool = postgres_context().await?;
        let fixture = AcceptanceFixture::new(pool, "product-total");
        let _timer = fixture.timer("total_cashback_for_product_sums_records_across_customers");

        let groceries = fixture.register_category("Groceries", dec!(0.02)).await?;
        let fuel = fixture.register_category("Fuel", dec!(0.01)).await?;
        let market_a = fixture.register_merchant("Market-A", true).await?;
        let market_b = fixture.register_merchant("Market-B", true).await?;
        let fuel_co = fixture.register_merchant("FuelCo", true).await?;

        fixture
            .record_purchase("cust-001", &market_a, &groceries, dec!(120.00))
            .await?;
        fixture
            .record_purchase("cust-002", &market_b, &groceries, dec!(80.00))
            .await?;
        fixture
            .record_purchase("cust-003", &fuel_co, &fuel, dec!(500.00))
            .await?;

        let total = fixture.product_total(&groceries).await?;
        assert_eq!(total["totalCashback"].to_string(), "4.00");
        assert_eq!(total["recordCount"], 2);

        Ok::<(), Box<dyn std::error::Error>>(())
    })
}

#[test]
fn product_total_returns_zero_count_when_empty() -> Result<(), Box<dyn std::error::Error>> {
    run_async(async {
        let pool = postgres_context().await?;
        let fixture = AcceptanceFixture::new(pool, "product-total-empty");
        let _timer = fixture.timer("product_total_returns_zero_count_when_empty");
        let category = fixture.category_without_registration("Travel");

        let total = fixture.product_total(&category).await?;
        assert_eq!(total["product"], category.category.name);
        assert_eq!(total["totalCashback"].to_string(), "0.00");
        assert_eq!(total["recordCount"], 0);

        Ok::<(), Box<dyn std::error::Error>>(())
    })
}
