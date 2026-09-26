use async_trait::async_trait;
use rust_decimal::Decimal;
use sqlx::{PgPool, Row};

use crate::application::port::{CashbackRepository, CategoryRepository, MerchantRepository, PortError};
use crate::domain::model::{CashbackRecord, Merchant, ProductCategory};

fn normalize(name: &str) -> String {
    name.trim().to_lowercase()
}

fn db_error(error: sqlx::Error) -> PortError {
    PortError::Database(error.to_string())
}

#[derive(Clone)]
pub struct PgMerchantRepository {
    pool: PgPool,
}

impl PgMerchantRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl MerchantRepository for PgMerchantRepository {
    async fn save(&self, merchant: &Merchant) -> Result<(), PortError> {
        sqlx::query(
            r#"INSERT INTO merchant (normalized_name, name, partner) VALUES ($1, $2, $3)"#,
        )
        .bind(normalize(&merchant.name))
        .bind(&merchant.name)
        .bind(merchant.partner)
        .execute(&self.pool)
        .await
        .map_err(db_error)?;

        Ok(())
    }

    async fn find_by_name(&self, name: &str) -> Result<Option<Merchant>, PortError> {
        let row = sqlx::query("SELECT name, partner FROM merchant WHERE normalized_name = $1")
            .bind(normalize(name))
            .fetch_optional(&self.pool)
            .await
            .map_err(db_error)?;

        Ok(row.map(|row| Merchant {
            name: row.get("name"),
            partner: row.get("partner"),
        }))
    }
}

#[derive(Clone)]
pub struct PgCategoryRepository {
    pool: PgPool,
}

impl PgCategoryRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CategoryRepository for PgCategoryRepository {
    async fn save(&self, category: &ProductCategory) -> Result<(), PortError> {
        sqlx::query(
            r#"INSERT INTO product_category (mcc, name, cashback_rate) VALUES ($1, $2, $3)"#,
        )
        .bind(&category.mcc)
        .bind(&category.name)
        .bind(category.cashback_rate)
        .execute(&self.pool)
        .await
        .map_err(db_error)?;

        Ok(())
    }

    async fn save_default_rate(&self, rate: Decimal) -> Result<(), PortError> {
        sqlx::query(
            r#"INSERT INTO default_cashback_rate (id, rate) VALUES (1, $1)
               ON CONFLICT (id) DO UPDATE SET rate = EXCLUDED.rate"#,
        )
        .bind(rate)
        .execute(&self.pool)
        .await
        .map_err(db_error)?;

        Ok(())
    }

    async fn default_rate(&self) -> Result<Option<Decimal>, PortError> {
        Ok(sqlx::query("SELECT rate FROM default_cashback_rate WHERE id = 1")
            .fetch_optional(&self.pool)
            .await
            .map_err(db_error)?
            .map(|row| row.get("rate")))
    }

    async fn find_by_mcc(&self, mcc: &str) -> Result<Option<ProductCategory>, PortError> {
        Ok(sqlx::query("SELECT mcc, name, cashback_rate FROM product_category WHERE mcc = $1")
            .bind(mcc)
            .fetch_optional(&self.pool)
            .await
            .map_err(db_error)?
            .map(|row| ProductCategory {
                mcc: row.get("mcc"),
                name: row.get("name"),
                cashback_rate: row.get("cashback_rate"),
            }))
    }
}

#[derive(Clone)]
pub struct PgCashbackRepository {
    pool: PgPool,
}

impl PgCashbackRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl CashbackRepository for PgCashbackRepository {
    async fn save(&self, record: &CashbackRecord) -> Result<(), PortError> {
        sqlx::query(
            r#"INSERT INTO cashback_record (customer_id, merchant_name, product_category, cashback_amount)
               VALUES ($1, $2, $3, $4)"#,
        )
        .bind(&record.customer_id)
        .bind(&record.merchant_name)
        .bind(&record.product_category)
        .bind(record.cashback_amount)
        .execute(&self.pool)
        .await
        .map_err(db_error)?;

        Ok(())
    }

    async fn find_by_customer_id(&self, customer_id: &str) -> Result<Vec<CashbackRecord>, PortError> {
        let rows = sqlx::query(
            r#"SELECT customer_id, merchant_name, product_category, cashback_amount
               FROM cashback_record
               WHERE customer_id = $1
               ORDER BY id"#,
        )
        .bind(customer_id)
        .fetch_all(&self.pool)
        .await
        .map_err(db_error)?;

        Ok(rows
            .into_iter()
            .map(|row| CashbackRecord {
                customer_id: row.get("customer_id"),
                merchant_name: row.get("merchant_name"),
                product_category: row.get("product_category"),
                cashback_amount: row.get("cashback_amount"),
            })
            .collect())
    }

    async fn total_for_product_category(&self, product_category: &str) -> Result<Decimal, PortError> {
        let row = sqlx::query(
            "SELECT COALESCE(SUM(cashback_amount), 0) AS total FROM cashback_record WHERE product_category = $1",
        )
        .bind(product_category)
        .fetch_one(&self.pool)
        .await
        .map_err(db_error)?;

        Ok(row.get("total"))
    }

    async fn count_for_product_category(&self, product_category: &str) -> Result<i64, PortError> {
        let row = sqlx::query(
            "SELECT COUNT(*) AS count FROM cashback_record WHERE product_category = $1",
        )
        .bind(product_category)
        .fetch_one(&self.pool)
        .await
        .map_err(db_error)?;

        Ok(row.get("count"))
    }
}
