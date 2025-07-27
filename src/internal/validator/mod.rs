use crate::internal::serializer::ResponseCode;

pub trait Validatable {
    fn validate(&self) -> Result<(), ResponseCode>;
}
