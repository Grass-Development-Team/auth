use crate::types::ValidatorError;

pub trait Validate {
    fn validate(&self) -> Result<(), ValidatorError>;
}