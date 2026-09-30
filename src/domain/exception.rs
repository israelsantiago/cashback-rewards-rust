use thiserror::Error;

#[derive(Debug, Error)]
pub enum DomainError {
    #[error("merchant '{0}' is already registered")]
    MerchantAlreadyRegistered(String),
    #[error("merchant '{0}' is not registered")]
    MerchantNotFound(String),
    #[error("default cashback rate is not configured")]
    DefaultRateNotConfigured,
}
