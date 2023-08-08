use crate::content_type::ContentType;
use std::io::{prelude::*, BufReader, Error, Result};
use std::str;
use std::{collections::HashMap, net::TcpStream};

pub struct Part {
    pub header: MIME_Header,
    pub disposition: String,
    pub disposition_params: HashMap<String, String>,
    pub content_type: ContentType,
    pub body: Vec<u8>,
}

impl Part {
    fn new() -> Self {
        Self {
            header: MIME_Header::new(),
            disposition: "".to_string(),
            disposition_params: HashMap::new(),
            content_type: ContentType::None,
            body: Vec::new(),
        }
    }
}

// Reader is an iterator over parts in a MIME multipart body.
// Reader's underlying parser consumes its input as needed. Seeking isn't supported.
pub struct Reader {
    content_length: usize,
    pub current_part: Part,
    parts_read: usize,
    bytes_read: usize,
    remaining_bytes: Vec<u8>,

    nl: Vec<u8>,                 // "\r\n" or "\n" (set after seeing first boundary line)
    nl_dash_boundary: Vec<u8>,   // nl + "--boundary"
    dash_boundary_dash: Vec<u8>, // "--boundary--"
    dash_boundary: Vec<u8>,      // "--boundary"
}

impl Reader {
    pub fn new(boundary: &str, content_length: usize) -> Self {
        // b := []byte("\r\n--" + boundary + "--")
        let mut b = Vec::new();
        b.extend_from_slice(b"\r\n--");
        b.extend_from_slice(boundary.as_bytes());
        b.extend_from_slice(b"--");

        Self {
            content_length,
            current_part: Part::new(),
            bytes_read: 0,
            remaining_bytes: Vec::new(),
            parts_read: 0,
            nl: (&b[..2]).to_vec(),
            nl_dash_boundary: (&b[..boundary.len() + 2]).to_vec(),
            dash_boundary_dash: (&b[2..]).to_vec(),
            dash_boundary: (&b[2..b.len() - 2]).to_vec(),
        }
    }

    pub fn next(&mut self, reader: &mut BufReader<TcpStream>) -> Option<&Part> {
        let mut buf = vec![0; 8192];

        // boundary
        if let Ok(n) = reader.read_until(b'\n', &mut buf) {
            self.bytes_read += n;
            let mut v = (&buf[..n]).to_vec();
            if v.ends_with(b"\r") {
                v.pop();
            }
            if v == self.dash_boundary_dash {
                println!("MultiPart end");
                return None;
            }
            if v != self.dash_boundary {
                return None;
            }
        } else {
            return None;
        }

        let mut content_disposition = "";
        // Content-Disposition
        if let Ok(n) = reader.read_until(b'\n', &mut buf) {
            self.bytes_read += n;
            if buf.starts_with(b"Content-Disposition") {
                content_disposition = str::from_utf8(&buf[..n]).unwrap().trim();
            } else {
                return None;
            }
        } else {
            return None;
        }

        let mut content_type_str = "";
        // Content-Type
        if let Ok(n) = reader.read_until(b'\n', &mut buf) {
            self.bytes_read += n;
            if buf.starts_with(b"Content-Type") {
                content_type_str = str::from_utf8(&buf[..n]).unwrap().trim();

                //Skip next blank line
                match reader.read_until(b'\n', &mut buf) {
                    Ok(n) => {
                        if n == 0 {
                            return None;
                        }
                        self.bytes_read += n;
                    }
                    Err(err) => {
                        println!("Content-Type in MultiPart error: {}", err);
                        return None;
                    }
                }
            }
        } else {
            return None;
        }

        // Body
        loop {
            match reader.read(&mut buf) {
                Ok(0) => {
                    println!("MultiPart end");
                    break;
                }
                Ok(n) => {
                    println!("MultiPart read: {n}");
                    self.process_part(&buf[0..n]);
                    // part.content_type = self.current_part.content_type;
                    return Some(&self.current_part);
                }
                Err(err) => {
                    println!("MultiPart error: {}", err)
                }
            }
        }

        None
    }

    pub fn body(&mut self, bufreader: &mut BufReader<TcpStream>) -> Result<Vec<u8>> {
        if self.bytes_read >= self.content_length {
            return Ok(vec![]);
        }
        let mut buf = vec![0; 1024];
        let mut result = Vec::new();

        loop {
            match bufreader.read(&mut buf) {
                Ok(n) => {
                    if n <= 0 {
                        return Ok(result);
                    }

                    let data = &buf[..n];
                    let mut lfs = Vec::new();
                    for (i, c) in data.iter().enumerate() {
                        if c == &b'\n' {
                            if i > 0 && data[i - 1] == b'\r' {
                                lfs.push(i);
                            }
                        }
                    }

                    // match nl-dash-boundary
                    let mut matched_index = usize::MAX;
                    for &i in lfs.iter() {
                        let end = i + (self.nl_dash_boundary.len());

                        if end < data.len() && &data[i..end] == self.nl_dash_boundary {
                            matched_index = i;
                        }
                    }

                    if matched_index != usize::MAX {
                        result.extend_from_slice(&data[..matched_index]);
                        // store remaining bytes
                        self.remaining_bytes = (&data[matched_index..]).to_vec();
                    } else {
                        result.extend_from_slice(data);
                    }

                    self.bytes_read += n;
                }
                Err(err) => {
                    return Err(err);
                }
            }
        }

        // Err(Error::new(std::io::ErrorKind::BrokenPipe, "Broken pipe"))
    }

    /*
    POST /foo HTTP/1.1
    Content-Length: 68137
    Content-Type: multipart/form-data; boundary=---------------------------974767299852498929531610575

    -----------------------------974767299852498929531610575
    Content-Disposition: form-data; name="description"

    some text
    -----------------------------974767299852498929531610575
    Content-Disposition: form-data; name="myFile"; filename="foo.txt"
    Content-Type: text/plain

    (content of the uploaded file foo.txt)
    -----------------------------974767299852498929531610575--
    */
    fn process_part(&mut self, buf: &[u8]) {
        // println!(
        //     "process_part: \n{}",
        //     String::from_utf8(buf.to_vec()).unwrap()
        // );
        let parts: Vec<&[u8]> = buf.split(|&b| b == b'\n').collect();

        // println!("Parts content--");
        // for part in parts.clone() {
        //     println!("{}", String::from_utf8(part.to_vec()).unwrap());
        // }
        // println!("Parts content--");

        let mut iter = parts.iter();
        if let Some(&line_boundary) = iter.next() {
            let mut line_boundary = line_boundary.to_vec();
            if line_boundary.ends_with(b"\r") {
                line_boundary.pop();
            }
            let line_boundary = &line_boundary[..];

            if line_boundary == self.dash_boundary {
                println!("Yes");
                if let Some(&line_content_disposition) = iter.next() {
                    self.handle_content_disposition(line_content_disposition);
                    if let Some(&line_content_type) = iter.next() {
                        let mut is_value = true;
                        let line_content_type_str = String::from_utf8(line_content_type.to_vec())
                            .unwrap()
                            .trim()
                            .to_string();
                        if line_content_type_str.len() > 0 {
                            self.current_part.content_type =
                                ContentType::parse(&line_content_type_str);
                            iter.next().unwrap();
                            is_value = false;
                        }

                        let mut i = 0;
                        while let Some(data) = iter.next() {
                            if data.starts_with(&self.dash_boundary) {
                                break;
                            }
                            if i > 0 {
                                self.current_part.body.extend_from_slice(b"\n");
                            }
                            self.current_part.body.extend_from_slice(data);
                            i += 1;
                        }

                        if is_value {
                            let part: &mut Part = &mut self.current_part;
                            part.disposition_params.insert(
                                (&part.disposition[..]).to_string(),
                                String::from_utf8((&part.body[..]).to_vec()).unwrap(),
                            );
                        }
                    }
                }
            }
        }
    }

    fn handle_content_disposition(&mut self, content_disposition: &[u8]) {
        if content_disposition.starts_with(b"Content-Disposition") {
            let str = String::from_utf8(content_disposition.to_vec()).unwrap();
            let v: Vec<&str> = str.split(";").collect();
            if v.len() >= 2 {
                self.current_part.disposition = v[1].trim().to_string();
            } else {
                self.current_part.disposition = "".to_string();
            }
        }
    }
}

// impl Iterator for Reader {
//     type Item = Part;
//
// }

type MIME_Header = HashMap<String, Vec<String>>;
