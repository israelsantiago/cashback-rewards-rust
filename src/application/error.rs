use thiserror::Error;

use crate::application::port::out::PortError;
use crate::domain::exception::DomainError;

#[derive(Debug, Error)]
pub enum ApplicationError {
    #[error(transparent)]
    Domain(#[from] DomainError),
    #[error(transparent)]
    Port(#[from] PortError),
}
