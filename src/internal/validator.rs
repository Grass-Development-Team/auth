pub trait Validatable<T> {
    fn validate(&self) -> Result<(), T>;
}
