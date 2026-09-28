use rust_decimal::{Decimal, RoundingStrategy};

pub struct CashbackCalculator;

impl CashbackCalculator {
    pub fn calculate(purchase_amount: Decimal, cashback_rate: Decimal) -> Decimal {
        (purchase_amount * cashback_rate)
            .round_dp_with_strategy(2, RoundingStrategy::MidpointNearestEven)
    }
}
