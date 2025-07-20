use super::common::Response;

impl<T> Response<T> {
    pub fn new_error(code: u16, message: String) -> Self {
        Response {
            code,
            message,
            data: None,
        }
    }

    pub fn is_err(&self) -> bool {
        self.code != 200
    }
}
