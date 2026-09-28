use rust_decimal::Decimal;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MinimumPurchaseThreshold {
    pub amount: Decimal,
}

impl Default for MinimumPurchaseThreshold {
    fn default() -> Self {
        Self {
            amount: Decimal::new(100, 2),
        }
    }
}

impl MinimumPurchaseThreshold {
    pub fn is_met_by(&self, purchase_amount: Decimal) -> bool {
        purchase_amount >= self.amount
    }
}
