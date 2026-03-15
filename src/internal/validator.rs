use crate::internal::error::AppError;

pub trait Validatable<T = AppError> {
    fn validate(&self) -> Result<(), T>;
}
