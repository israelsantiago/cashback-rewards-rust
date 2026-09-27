use rust_decimal::{Decimal, RoundingStrategy};

pub fn calculate_cashback(purchase_amount: Decimal, cashback_rate: Decimal) -> Decimal {
    (purchase_amount * cashback_rate)
        .round_dp_with_strategy(2, RoundingStrategy::MidpointNearestEven)
}
