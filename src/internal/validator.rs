use crate::internal::serializer::ResponseCode;

pub trait Validatable<T = ResponseCode> {
    fn validate(&self) -> Result<(), T>;
}
