use crate::support::{
    acceptance::AcceptanceFixture, postgres::postgres_context, runtime::run_async,
};
use rust_decimal::dec;
#[test]
fn category_rates_cover_groceries_and_fuel() -> Result<(), Box<dyn std::error::Error>> {
    run_async(async {
        let pool = postgres_context().await?;
        let fixture = AcceptanceFixture::new(pool, "category-rates");
        let _timer = fixture.timer("category_rates_cover_groceries_and_fuel");

        let groceries = fixture.register_category("Groceries", dec!(0.02)).await?;
        let fuel = fixture.register_category("Fuel", dec!(0.01)).await?;
        let grocer = fixture.register_merchant("Grocer", true).await?;
        let fuel_co = fixture.register_merchant("FuelCo", true).await?;
        fixture
            .record_purchase("cust-g", &grocer, &groceries, dec!(100.00))
            .await?;
        fixture
            .record_purchase("cust-f", &fuel_co, &fuel, dec!(100.00))
            .await?;

        let g = fixture.cashback_for("cust-g").await?;
        let f = fixture.cashback_for("cust-f").await?;
        assert_eq!(g[0]["cashbackAmount"].to_string(), "2.00");
        assert_eq!(f[0]["cashbackAmount"].to_string(), "1.00");

        Ok::<(), Box<dyn std::error::Error>>(())
    })
}

#[test]
fn unmapped_mcc_uses_configured_default_rate() -> Result<(), Box<dyn std::error::Error>> {
    run_async(async {
        let pool = postgres_context().await?;
        let mut fixture = AcceptanceFixture::new(pool, "default-rate");
        let _timer = fixture.timer("unmapped_mcc_uses_configured_default_rate");

        fixture.configure_default_rate(dec!(0.005)).await?;
        let merchant = fixture.register_merchant("CityPharmacy", true).await?;
        let unknown = fixture.identity().mcc("unmapped");
        fixture
            .record_purchase_with_mcc("cust-default", &merchant, &unknown, dec!(100.00))
            .await?;

        let records = fixture.cashback_for("cust-default").await?;
        assert_eq!(records.as_array().map(Vec::len), Some(1));
        assert_eq!(records[0]["productCategory"], "Other");
        assert_eq!(records[0]["cashbackAmount"].to_string(), "0.50");

        Ok::<(), Box<dyn std::error::Error>>(())
    })
}
