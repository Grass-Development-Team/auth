use super::common::Response;

impl<T> Response<T> {
    pub fn new_error(code: u16, message: String) -> Self {
        Response {
            code,
            message,
            data: None,
        }
    }
}