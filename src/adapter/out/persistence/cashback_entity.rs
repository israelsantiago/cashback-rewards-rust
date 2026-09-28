use rust_decimal::Decimal;
use sqlx::FromRow;

use crate::domain::model::CashbackRecord;

#[derive(Debug, FromRow)]
pub(crate) struct CashbackRecordEntity {
    pub(crate) customer_id: String,
    pub(crate) merchant_name: String,
    pub(crate) product_category: String,
    pub(crate) cashback_amount: Decimal,
}

impl CashbackRecordEntity {
    pub(crate) fn from_domain(record: &CashbackRecord) -> Self {
        Self {
            customer_id: record.customer_id.clone(),
            merchant_name: record.merchant_name.clone(),
            product_category: record.product_category.clone(),
            cashback_amount: record.cashback_amount,
        }
    }

    pub(crate) fn into_domain(self) -> CashbackRecord {
        CashbackRecord {
            customer_id: self.customer_id,
            merchant_name: self.merchant_name,
            product_category: self.product_category,
            cashback_amount: self.cashback_amount,
        }
    }
}
