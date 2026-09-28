use rust_decimal::Decimal;
use sqlx::FromRow;

use crate::domain::model::ProductCategory;

#[derive(Debug, FromRow)]
pub(crate) struct ProductCategoryEntity {
    pub(crate) mcc: String,
    pub(crate) name: String,
    pub(crate) cashback_rate: Decimal,
}

impl ProductCategoryEntity {
    pub(crate) fn from_domain(category: &ProductCategory) -> Self {
        Self {
            mcc: category.mcc.clone(),
            name: category.name.clone(),
            cashback_rate: category.cashback_rate,
        }
    }

    pub(crate) fn into_domain(self) -> ProductCategory {
        ProductCategory {
            mcc: self.mcc,
            name: self.name,
            cashback_rate: self.cashback_rate,
        }
    }
}
