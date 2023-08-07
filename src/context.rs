use crate::request::Request;
use crate::response::Response;
use std::fs;
use std::io::{prelude::*, BufReader, Error};
use std::{cell::RefCell, net::TcpStream, rc::Rc};

pub struct Context {
    pub request: Request,
    pub stream: TcpStream,
}

impl Context {
    pub fn new(stream: TcpStream) -> Result<Context, Error> {
        let stream_clone = stream.try_clone().unwrap();
        let reader: BufReader<TcpStream> = BufReader::new(stream_clone);
        let mut request = Request::new(reader);

        return match request.init() {
            Ok(()) => Result::Ok(Context { request, stream }),
            Err(err) => Result::Err(err),
        };
    }
}

pub trait ContextFn {
    fn json(&mut self, content: &str);
    fn error(&mut self);
    fn error_with_status(&mut self, code: u32, reason: String);
}

impl ContextFn for Context {
    fn json(&mut self, content: &str) {
        let mut response = Response::build();

        response.add_header("Content-Type", "application/json");
        response.add_header("Content-Length", &format!("{}", content.len()));

        let response = format!(
            "{}\r\n{}\r\n\r\n{}",
            response.get_status_line(),
            response.get_header(),
            content
        );

        let ret = self.stream.write_all(response.as_bytes());
        match ret {
            Ok(()) => (),
            Err(err) => println!("Response error: {}", err),
        }
    }

    fn error(&mut self) {
        let mut response = Response::build_error(404, "NOT FOUND".to_string());
        let content = fs::read_to_string("404.html").unwrap();
        response.add_header("Content-Length", &format!("{}", content.len()));
        let response = format!(
            "{}\r\n{}\r\n\r\n{}",
            response.get_status_line(),
            response.get_header(),
            content
        );

        self.stream.write_all(response.as_bytes()).unwrap();
    }

    fn error_with_status(&mut self, code: u32, reason: String) {
        let mut response = Response::build_error(code, reason);
        let content = "{\"code\":-1,\"message\":\"method error\"}";
        response.add_header("Content-Type", "application/json");
        response.add_header("Content-Length", &format!("{}", content.len()));
        let response = format!(
            "{}\r\n{}\r\n\r\n{}",
            response.get_status_line(),
            response.get_header(),
            content
        );

        self.stream.write_all(response.as_bytes()).unwrap();
    }
}
