use std::{
    collections::HashMap,
    io::{prelude::*, BufReader, Error, ErrorKind, Result},
    net::TcpStream,
    str,
};

use crate::content_type::ContentType;
use crate::request::mime::multipart::{MultiPart, Part};

/*
POST /hello HTTP/1.1
Host: www.example.com
User-Agent: Mozilla/5.0
Content-Type: application/x-www-form-urlencoded
Content-Length: 18

name=John&Doe&age=30
*/

const MAX_PARSE_BODY_SIZE: usize = 10 << 20; // 10MB

const METHODS: &[&str] = &[
    "GET", "POST", "PUT", "DELETE", "OPTIONS", "HEAD", "TRACE", "CONNECT",
];

const PROTOCOLS: &[&str] = &["HTTP/1.0", "HTTP/1.1", "HTTP/2"];

pub struct Request {
    pub method: String,
    pub uri: String,
    pub path: String,
    pub version: String,
    pub headers: HashMap<String, Vec<String>>,
    pub content_type: ContentType,
    pub content_length: usize,
    boundary: String,
    pub body: Vec<u8>,
    pub form: HashMap<String, Vec<String>>,
    pub post_form: HashMap<String, Vec<String>>,
    pub reader: BufReader<TcpStream>,
    multipart: MultiPart,
}

pub mod mime;

impl Request {
    pub fn header(&self, key: &str) -> Vec<String> {
        self.headers.get(key).unwrap_or(&Vec::new()).to_vec()
    }

    pub fn header_first(&self, key: &str) -> String {
        match self.headers.get(key) {
            Some(values) if values.len() > 0 => values[0].to_string(),
            _ => "".to_string(),
        }
    }

    pub fn new(reader: BufReader<TcpStream>) -> Request {
        Request {
            method: "".to_string(),
            uri: "".to_string(),
            path: "".to_string(),
            version: "".to_string(),
            headers: HashMap::new(),
            content_type: ContentType::None,
            content_length: 0,
            boundary: "".to_string(),
            body: vec![],
            form: HashMap::new(),
            post_form: HashMap::new(),
            reader,
            multipart: MultiPart::new("", 0),
        }
    }

    pub fn init(&mut self) -> Result<()> {
        // Read the request line
        let mut request_line = String::new();
        match self.reader.read_line(&mut request_line) {
            Ok(n) if n > 0 => (),
            _ => {
                return Err(Error::new(
                    ErrorKind::ConnectionRefused,
                    "No continious request",
                ))
            }
        }

        if request_line.is_empty() {
            return Err(Error::new(ErrorKind::Unsupported, "request line is empty"));
        }

        /*
        let reg: Regex = Regex::new(r"^(GET|POST|PUT|DELETE|OPTIONS|HEAD|TRACE|CONNECT) [^\s]+ HTTP/1\.[01]").unwrap();
        if !reg.is_match(&request_line) {
            return Err(Error::new(ErrorKind::Unsupported, "Not a HTTP request!"));
        }
        */

        let (method, uri, path, queries, version) = parse_request_line(&request_line);
        if !METHODS.contains(&&method[..]) {
            return Err(Error::new(
                ErrorKind::Unsupported,
                "Unsupported HTTP METHOD",
            ));
        }

        if !PROTOCOLS.contains(&&version[..]) {
            return Err(Error::new(
                ErrorKind::Unsupported,
                "Unsupported HTTP VERSION",
            ));
        }

        // Println request line
        println!("\n{}", request_line.trim());

        // Read the headers
        let mut header = String::new();
        loop {
            let mut line = String::new();
            self.reader.read_line(&mut line).unwrap();

            if line.trim().is_empty() {
                break;
            }

            // Println headers
            println!("{}", line.trim());

            header.push_str("\n");
            header.push_str(&line);
        }

        let headers = parse_request_header(&header);
        // println!("{request_line}");
        // if queries.len() > 0 {
        //     println!("{:#?}", queries);
        // }
        // println!("{:#?}", headers);
        println!();

        self.method = method;
        self.uri = uri;
        self.path = path;
        self.version = version;
        self.form = queries;
        self.headers = headers;
        self.content_type = ContentType::parse(&&self.header_first("Content-Type"));
        self.content_length = self.header_first("Content-Length").parse().unwrap_or(0);

        match &self.content_type {
            ContentType::MultiPart {
                sub_type: _,
                boundary,
            } => self.boundary = boundary.clone(),
            _ => (),
        }

        // println!("Content-Type: {:?}", self.content_type);

        Ok(())
    }

    pub fn body(&mut self) -> Vec<u8> {
        let length = self.content_length;
        if length <= 0 {
            return vec![];
        }

        if length > MAX_PARSE_BODY_SIZE {
            println!("Body too large!");
            return vec![];
        }

        if self.body.len() > 0 {
            return self.body.to_vec();
        }

        let mut buf = [0; 4096];
        let mut bytes_remaining = length;

        while bytes_remaining > 0 {
            match self.reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    self.body.extend_from_slice(&buf[0..n]);
                    bytes_remaining -= n;
                }
                Err(e) => {
                    eprintln!("Error while reading from stream: {}", e);
                    break;
                }
            }
        }

        // println!("content length: {length}");
        // println!("bytes remaining: {bytes_remaining}");
        // println!("body length: {}", self.body.len());

        self.body.to_vec()
    }

    pub fn read_body(&mut self, buf: &mut Vec<u8>) -> Result<usize> {
        self.reader.read(buf)
    }

    pub fn parse_post_form(&mut self) {
        let body = if self.body.len() > 0 {
            self.body.to_vec()
        } else {
            self.body()
        };

        match String::from_utf8(body) {
            Ok(content) => parse_form(&content, &mut self.post_form),
            Err(e) => println!("Error occured when parse_post_form: {}", e),
        }
    }

    pub fn multipart<'a>(&'a mut self) -> &'a mut Self {
        self.multipart = MultiPart::new(&self.boundary, self.content_length);
        self
    }

    pub fn next(&mut self) -> Option<&Part> {
        self.multipart.next(&mut self.reader)
    }

    pub fn part_body(&mut self) -> Result<Vec<u8>> {
        let multipart_reader = &mut self.multipart;
        multipart_reader.body(&mut self.reader)
    }
}

//POST /hello?name=sfdex&age=18 HTTP/1.1
fn parse_request_line(
    request_line: &str,
) -> (String, String, String, HashMap<String, Vec<String>>, String) {
    let v: Vec<&str> = request_line.split_whitespace().collect();

    let uri = v[1];
    let mut queries: HashMap<String, Vec<String>> = HashMap::new();
    let mut path = uri.to_string();

    // parse queries
    if let Some((path_in_uri, queries_str)) = uri.split_once("?") {
        path = path_in_uri.to_string();
        parse_form(queries_str, &mut queries);
    }

    (
        v[0].to_string(),
        uri.to_string(),
        path,
        queries,
        v[2].to_string(),
    )
}

fn parse_request_header(request_header: &str) -> HashMap<String, Vec<String>> {
    let mut headers = request_header.lines();
    headers.next(); // skip the first empty line

    let mut headers_map: HashMap<String, Vec<String>> = HashMap::new();
    for header in headers {
        let mut parts = header.splitn(2, ":");
        if let (Some(k), Some(v)) = (parts.next(), parts.next()) {
            let values = headers_map.entry(k.to_string()).or_insert(vec![]);
            values.push(v.trim().to_string());
        }
    }

    headers_map
}

fn parse_form(forms: &str, map: &mut HashMap<String, Vec<String>>) {
    for pair in forms.split("&") {
        let mut parts = pair.splitn(2, "=");

        if let (Some(k), Some(v)) = (parts.next(), parts.next()) {
            let values = map.entry(k.to_string()).or_insert(vec![]);
            values.push(v.trim().to_string());
        }
    }
}
