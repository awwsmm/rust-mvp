use std::collections::HashMap;

pub struct Request {
    pub request_line: String,
    pub headers: HashMap<String, String>,
    pub body: Option<String>,
}

impl Request {
    pub fn new(
        request_line: String,
        headers: HashMap<String, String>,
        body: Option<String>,
    ) -> Request {
        Request {
            request_line,
            headers,
            body,
        }
    }
}
