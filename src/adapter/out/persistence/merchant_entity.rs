use sqlx::FromRow;

use crate::domain::model::Merchant;

#[derive(Debug, FromRow)]
pub(crate) struct MerchantEntity {
    pub(crate) normalized_name: String,
    pub(crate) name: String,
    pub(crate) partner: bool,
}

impl MerchantEntity {
    pub(crate) fn from_domain(merchant: &Merchant) -> Self {
        Self {
            normalized_name: normalize(&merchant.name),
            name: merchant.name.clone(),
            partner: merchant.partner,
        }
    }

    pub(crate) fn into_domain(self) -> Merchant {
        Merchant {
            name: self.name,
            partner: self.partner,
        }
    }
}

pub(crate) fn normalize(name: &str) -> String {
    name.trim().to_lowercase()
}
