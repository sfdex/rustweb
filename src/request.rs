use std::{
    cell::RefCell,
    collections::HashMap,
    io::{prelude::*, BufReader, Error, ErrorKind},
    net::TcpStream,
    rc::Rc,
    str,
};

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
    pub queries: HashMap<String, String>,
    pub headers: HashMap<String, String>,
    stream: Rc<RefCell<TcpStream>>,
    remaining: Vec<u8>,
}

pub mod mime;

impl Request {
    pub fn header(&self, key: &str) -> String {
        self.headers
            .get(key)
            .unwrap_or(&String::from(""))
            .to_string()
    }

    pub fn new(stream: Rc<RefCell<TcpStream>>) -> Result<Request, Error> {
        let mut binding = stream.borrow_mut();
        let mut reader = BufReader::new(&mut *binding);

        // Read the request line
        let mut request_line = String::new();
        reader.read_line(&mut request_line).unwrap();
        println!("Request line: {request_line}");
        if request_line.is_empty() {
            return Err(Error::new(ErrorKind::Unsupported, "request line is empty"));
        }

        // Read the headers
        let mut header = String::new();
        loop {
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();

            if line.trim().is_empty() {
                break;
            }

            header.push_str("\n");
            header.push_str(&line);
        }

        let (method, path, queries, version) = parse_request_line(&request_line);
        let headers = parse_request_header(&header);
        println!("{request_line}");
        if queries.len() > 0 {
            println!("{:#?}", queries);
        }
        println!("{:#?}", headers);
        println!();

        let remaining = reader.buffer();
        // println!("remaining len: {}",remaining.len());

        Ok(Request {
            method,
            path,
            version,
            queries,
            headers,
            stream: Rc::clone(&stream),
            remaining: remaining.to_vec(),
        })
    }

    pub fn body(&mut self) -> Vec<u8> {
        let mut result = self.remaining.to_vec();
        let length: usize = self.header("Content-Length").parse().unwrap_or(0);
        if length <= 0 {
            return result;
        }

        let mut buf = [0; 4096];
        let mut bytes_remaining = length - result.len();

        while bytes_remaining > 0 {
            match self.stream.borrow_mut().read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    result.extend_from_slice(&buf[0..n]);
                    bytes_remaining -= n;
                }
                Err(e) => {
                    eprintln!("Error while reading from stream: {}", e);
                    break;
                }
            }
        }

        println!("content length: {length}");
        println!("bytes remaining: {bytes_remaining}");
        println!("body length: {}", result.len());

        result
    }
}

//POST /hello?name=sfdex&age=18 HTTP/1.1
fn parse_request_line(request_line: &str) -> (String, String, HashMap<String, String>, String) {
    let v: Vec<&str> = request_line.split_whitespace().collect();

    let uri = v[1];
    let mut queries = HashMap::new();
    let mut path = uri.to_string();

    // parse queries
    if let Some((path_in_uri, queries_str)) = uri.split_once("?") {
        path = path_in_uri.to_string();

        for pair in queries_str.split("&") {
            let mut parts = pair.splitn(2, "=");
            if let (Some(k), Some(v)) = (parts.next(), parts.next()) {
                queries.insert(k.to_string(), v.to_string());
            }
        }
    }

    (v[0].to_string(), path, queries, v[2].to_string())
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
