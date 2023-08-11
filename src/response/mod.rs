use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::io::Result;

use self::status::Status;
use crate::context::Context;

pub mod status;

pub struct Response {
    status: Status,
    header: HashMap<String, String>,

    body: Box<dyn Body>,
}

pub trait Body {
    fn get_content(&mut self, _buf: &mut Vec<u8>) -> Result<usize> {
        Ok(0)
    }

    fn provide_content(&self) -> &[u8] {
        &[0u8; 0]
    }

    fn get_content_type(&self) -> &str {
        ""
    }

    fn get_content_length(&self) -> Result<usize> {
        Ok(0)
    }

    fn get_content_disposition(&self) -> &str {
        ""
    }
}

pub struct NoneContent;

pub struct TextBody {
    pub content_type: String,
    pub content: Vec<u8>,
    pub cursor_index: usize,
}

pub struct JsonBody {
    content: Vec<u8>,
    cursor_index: usize,
}

pub struct FileBody {
    pub file: File,
    pub mime_type: &'static str,
    pub disposition: &'static str,
}

impl Body for NoneContent {}

impl TextBody {
    pub fn new(content_type: String, content: Vec<u8>) -> Self {
        Self {
            content_type,
            content,
            cursor_index: 0,
        }
    }
}

impl Body for TextBody {
    fn get_content(&mut self, buf: &mut Vec<u8>) -> Result<usize> {
        let buf_len = buf.len();
        let total = self.get_content_length().unwrap_or(0);
        let mut bytes = 0usize;
        let begin = self.cursor_index;
        for i in 0..buf_len {
            if self.cursor_index >= total {
                break;
            }

            buf[i] = self.content[begin + i];

            self.cursor_index += 1;
            bytes += 1;
        }
        Ok(bytes)
    }

    fn provide_content(&self) -> &[u8] {
        &self.content
    }

    fn get_content_type(&self) -> &str {
        &self.content_type
    }

    fn get_content_length(&self) -> Result<usize> {
        Ok(self.content.len())
    }
}

impl JsonBody {
    pub fn new(content: &[u8]) -> Self {
        Self {
            content: content.to_vec(),
            cursor_index: 0,
        }
    }
}

impl Body for JsonBody {
    fn get_content(&mut self, buf: &mut Vec<u8>) -> Result<usize> {
        let buf_len = buf.len();
        let total = self.get_content_length().unwrap_or(0);
        let mut bytes = 0usize;
        let begin = self.cursor_index;
        for i in 0..buf_len {
            if self.cursor_index >= total {
                break;
            }

            buf[i] = self.content[begin + i];

            self.cursor_index += 1;
            bytes += 1;
        }
        Ok(bytes)
    }

    fn get_content_type(&self) -> &str {
        "application/json"
    }

    fn get_content_length(&self) -> Result<usize> {
        Ok(self.content.len())
    }
}

impl Body for FileBody {
    fn get_content(&mut self, buf: &mut Vec<u8>) -> Result<usize> {
        self.file.read(buf)
    }

    fn get_content_type(&self) -> &str {
        self.mime_type
    }

    fn get_content_length(&self) -> Result<usize> {
        Ok(self.file.metadata()?.len() as usize)
    }

    fn get_content_disposition(&self) -> &str {
        self.disposition
    }
}

impl dyn Body {}

impl Response {
    pub fn new(status: Status, header: HashMap<String, String>, body: Box<dyn Body>) -> Self {
        Self {
            status,
            header,
            body,
        }
    }
    pub fn set_status(&mut self, status: Status) {
        self.status = status
    }

    pub fn set_header(&mut self, header: HashMap<String, String>) {
        self.header = header
    }

    pub fn add_header(&mut self, key: &str, value: &str) {
        self.header.insert(key.to_string(), value.to_string());
    }

    pub fn set_body(&mut self, body: Box<dyn Body>) {
        self.body = body
    }

    pub fn build() -> Response {
        Response {
            status: Status::OK,
            header: HashMap::new(),
            body: Box::new(NoneContent {}),
        }
    }

    pub fn build_error(status: Status) -> Response {
        Response {
            status,
            header: HashMap::new(),
            body: Box::new(NoneContent {}),
        }
    }

    pub fn get_status_line(&self) -> Vec<u8> {
        format!("HTTP/1.1 {}\r\n", self.status.to_string())
            .as_bytes()
            .to_vec()
    }

    pub fn get_header(&mut self) -> Vec<u8> {
        let mut headers = String::new();
        for (key, value) in &self.header {
            headers.push_str(&format!("{key}: {value}"));
            headers.push_str("\r\n");
        }
        let content_type = self.body.get_content_type();
        if !content_type.is_empty() {
            let content_length = self.body.get_content_length().unwrap_or(0);
            let content_disposition = self.body.get_content_disposition();
            headers.push_str(&format!("Content-Type: {}\r\n", content_type));
            headers.push_str(&format!("Content-Length: {}\r\n", content_length));
            if !content_disposition.is_empty() {
                headers.push_str(&format!("Content-Disposition: {}\r\n", content_disposition));
            }
        }

        headers.push_str("\r\n");
        headers.as_bytes().to_vec()
    }
}

impl Response {
    pub fn response(&mut self, context: &mut Context) -> Result<()> {
        let stream = &mut context.stream;
        stream.write(&self.get_status_line())?;
        stream.write(&self.get_header())?;

        // body
        let body = &mut self.body;
        let content_length = body.get_content_length().unwrap_or(0);
        if content_length > 0 {
            let mut buf = vec![0; 10];
            let mut writed = 0usize;
            loop {
                match body.get_content(&mut buf) {
                    Ok(n) => {
                        if n <= 0 {
                            break;
                        }
                        stream.write(&buf[..n])?;
                        writed += n;
                        if writed >= content_length {
                            break;
                        }
                    }
                    Err(err) => {
                        eprintln!("Error occured when response: {}", err);
                        break;
                    }
                }
            }
        }

        stream.flush()?;

        Ok(())
    }
}
