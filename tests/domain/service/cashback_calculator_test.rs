use cashback_rewards_rust::domain::service::CashbackCalculator;
use rust_decimal::dec;

#[test]
fn multiplies_purchase_amount_by_rate() {
    assert_eq!(
        CashbackCalculator::calculate(dec!(80.00), dec!(0.03)),
        dec!(2.40)
    );
}

#[test]
fn rounds_half_cent_tie_to_nearest_even_cent() {
    assert_eq!(
        CashbackCalculator::calculate(dec!(33.50), dec!(0.05)),
        dec!(1.68)
    );
}
