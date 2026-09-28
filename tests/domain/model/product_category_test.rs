use cashback_rewards_rust::domain::model::ProductCategory;
use rust_decimal::dec;

#[test]
fn unmapped_uses_default_rate_under_the_other_category() {
    let category = ProductCategory::unmapped("5912", dec!(0.005));
    assert_eq!(category.mcc, "5912");
    assert_eq!(category.name, "Other");
    assert_eq!(category.cashback_rate, dec!(0.005));
}
