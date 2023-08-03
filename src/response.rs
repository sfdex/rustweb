use std::collections::HashMap;
pub struct Response {
    status_code: u32,
    status_message: String,

    header: HashMap<String, String>,
    // body: [u8],
}

impl Response {
    pub fn set_status(&mut self, status_code: u32, status_message: String) {
        self.status_code = status_code;
        self.status_message = status_message;
    }

    pub fn set_header(&mut self, header: HashMap<String, String>) {
        self.header = header
    }

    pub fn add_header(&mut self, key: &str, value: &str) {
        self.header.insert(key.to_string(), value.to_string());
    }

    pub fn build() -> Response {
        Response {
            status_code: 0,
            status_message: String::from("OK"),
            header: HashMap::new(),
        }
    }

    pub fn build_error(code: u32, message: String) -> Response {
        Response {
            status_code: code,
            status_message: message,
            header: HashMap::new(),
        }
    }

    pub fn get_status_line(&self) -> String {
        format!("HTTP/1.1 {} {}", self.status_code, self.status_message)
    }

    pub fn get_header(&mut self) -> String {
        let len = self.header.len();
        let mut count = 0;

        let mut headers = String::new();
        for (key, value) in &self.header {
            headers.push_str(&format!("{key}: {value}"));
            if count < len - 1 {
                headers.push_str("\r\n");
            }
            count += 1;
        }

        headers
    }
}
