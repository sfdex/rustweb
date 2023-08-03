use std::{cell::RefCell, collections::HashMap, io::prelude::*, net::TcpStream, rc::Rc, str};

/*
POST /hello HTTP/1.1
Host: www.example.com
User-Agent: Mozilla/5.0
Content-Type: application/x-www-form-urlencoded
Content-Length: 18

name=John&Doe&age=30
*/

pub struct Request {
    pub method: String,
    pub path: String,
    pub version: String,
    pub headers: HashMap<String, String>,
    pub queries: HashMap<String, String>,
}

impl Request {
    pub fn header(&self, key: &str) -> String {
        self.headers
            .get(key)
            .unwrap_or(&String::from(""))
            .to_string()
    }

    pub fn new(mut stream: Rc<RefCell<TcpStream>>) -> Request {
        let mut buffer = [0; 2048];
        stream.borrow_mut().read(&mut buffer).unwrap();

        let request = str::from_utf8(&buffer).unwrap();

        let request_line_end = request.find("\r\n").unwrap_or(request.len());
        let (request_line, header_and_body) = request.split_at(request_line_end);
        let header_end = request.find("\r\n\r\n").unwrap_or(request.len());
        let (header, body) = header_and_body.split_at(header_end);

        let (method, path, version) = parse_request_line(request_line);
        let headers = parse_request_header(header);
        println!("New request:");
        println!("{request_line}");
        print!("headers:\n{:#?}", headers);

        Request {
            method,
            path: path.to_string(),
            version,
            headers,
            queries: HashMap::new(),
        }
    }
}

//POST /hello?name=sfdex&age=18 HTTP/1.1
fn parse_request_line(request_line: &str) -> (String, String, HashMap<String, String>, String) {
    let v: Vec<&str> = request_line.split_whitespace().collect();

    let uri = v[1].to_string();
    let queries = HashMap::new();
    // if uri.contains("?") {
    //     let queries_str:Vec<&str> = uri.split("?").collect();
    // }
    (v[0].to_string(), uri, queries, v[2].to_string())
}

fn parse_request_header(request_header: &str) -> HashMap<String, String> {
    let mut headers = request_header.lines();
    headers.next(); // skip the first empty line

    let mut headers_map = HashMap::new();
    for header in headers {
        let mut parts = header.splitn(2, ":");
        if let (Some(name), Some(value)) = (parts.next(), parts.next()) {
            headers_map.insert(name.trim().to_string(), value.trim().to_string());
        }
    }

    headers_map
}
