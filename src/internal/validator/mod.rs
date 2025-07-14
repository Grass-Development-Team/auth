use crate::internal::serializer::common::ResponseCode;

pub trait Validatable {
    fn validate(&self) -> Result<(), ResponseCode>;
}
