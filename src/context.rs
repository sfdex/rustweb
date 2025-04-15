use crate::request::Request;
use crate::response::status::Status;
use crate::response::{FileBody, JsonBody, NoneContent, Response};
use crate::Connection;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Error};
use std::net::TcpStream;
use std::time::Duration;

pub struct Context {
    pub request: Request,
    pub stream: TcpStream,
}

impl Context {
    pub fn new(conn: Connection) -> Result<Context, Error> {
        conn.stream
            .set_read_timeout(Some(Duration::from_secs(1)))
            .unwrap();
        let stream_clone = conn.stream.try_clone().unwrap();
        let reader: BufReader<TcpStream> = BufReader::new(stream_clone);
        let request = Request::new(reader, conn.address);

        let mut context = Context {
            request,
            stream: conn.stream,
        };

        match context.request.init() {
            Ok(()) => Ok(context),
            Err(err) => {
                // context.error();
                return Result::Err(err);
            }
        }
    }
}

pub trait ContextFn {
    fn ok(&mut self);
    fn json(&mut self, content: &[u8]);
    fn file(&mut self, file: File, filename: String);
    fn error(&mut self);
    fn error_with_status(&mut self, status: Status);
}

impl ContextFn for Context {
    fn ok(&mut self) {
        let content = b"{\"code\":200,\"message\":\"Upload finish!\"}";
        self.json(content);
    }

    fn json(&mut self, content: &[u8]) {
        let mut response =
            Response::new(Status::OK, HashMap::new(), Box::new(JsonBody::new(content)));
        response.response(self).unwrap();
    }

    fn file(&mut self, file: File, filename: String) {
        let disposition = format!("attachment; filename={}", filename);
        let body = FileBody::new(file, "application/octet-stream", disposition);
        let mut response = Response::new(Status::OK, HashMap::new(), Box::new(body));
        response.response(self).unwrap()
    }

    fn error(&mut self) {
        let body = FileBody {
            file: File::open("404.html").unwrap(),
            mime_type: "text/html",
            disposition: "".to_string(),
        };
        let mut response = Response::new(Status::NotFound, HashMap::new(), Box::new(body));

        response.response(self).unwrap();
    }

    fn error_with_status(&mut self, status: Status) {
        let mut response = Response::new(status, HashMap::new(), Box::new(NoneContent));
        response.response(self).unwrap();
    }
}
