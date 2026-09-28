use cashback_rewards_rust::domain::model::MinimumPurchaseThreshold;
use rust_decimal::dec;

#[test]
fn is_met_only_by_purchases_at_or_above_the_minimum() {
    let threshold = MinimumPurchaseThreshold::default();
    assert!(!threshold.is_met_by(dec!(0.50)));
    assert!(!threshold.is_met_by(dec!(0.99)));
    assert!(!threshold.is_met_by(dec!(0.999)));
    assert!(threshold.is_met_by(dec!(1.00)));
    assert!(threshold.is_met_by(dec!(25.00)));
}
