use std::{
    io::Write,
    net::TcpStream,
    rc::Rc, cell::RefCell, fs,
};
use crate::{response::Response, request::Request};

pub struct Context {
    pub request:Request,
    stream: Rc<RefCell<TcpStream>>,
}

pub trait ResponseFunc {
    fn json(&mut self, content: &str);
    fn error(&mut self);
}

impl Context {
    pub fn get_header(&self, key: &str) -> String {
        self.request.headers
            .get(key)
            .unwrap_or(&String::from(""))
            .to_string()
    }

    pub fn new(mut stream: TcpStream) -> Context {
        /* let buf_reader = BufReader::new(&stream);
        let http_request: Vec<String> = buf_reader
            .lines()
            .map(|result| result.unwrap())
            .take_while(|line| !line.is_empty())
            .collect();

        let mut proto_version = "";
        let mut path = "";
        let mut method = "";
        let mut request_header = HashMap::new();

        for (index, line) in http_request.iter().enumerate() {
            if index == 0 {
                for (index, text) in line.split_whitespace().enumerate() {
                    match index {
                        0 => method = text,
                        1 => path = text,
                        _ => proto_version = text,
                    }
                }
            } else {
                let mut iter = line.split(": ");
                request_header.insert(
                    iter.next().unwrap().to_string(),
                    iter.next().unwrap().to_string(),
                );
            }
        } */

        let stream = Rc::new(RefCell::new(stream));

        Context {
            request: Request::new(Rc::clone(&stream)),
            stream,
        }
    }
}

impl ResponseFunc for Context {
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
        
        self.stream.borrow_mut().write_all(response.as_bytes()).unwrap();
    }

    fn error(&mut self){
        let mut response = Response::build_error(404, "NOT FOUND".to_string());
        let content = fs::read_to_string("404.html").unwrap();
        response.add_header("Content-Length", &format!("{}", content.len()));
        let response = format!(
            "{}\r\n{}\r\n\r\n{}",
            response.get_status_line(),
            response.get_header(),
            content
        );
        
        self.stream.borrow_mut().write_all(response.as_bytes()).unwrap();
    }
}