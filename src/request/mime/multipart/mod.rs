use crate::content_type::ContentType;
use std::io::{prelude::*, BufReader, Result};
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
pub struct MultiPart {
    content_length: usize,
    pub current_part: Part,
    parts_read: usize,
    bytes_read: usize,
    remaining_bytes: Vec<u8>,
    is_end: bool,

    nl: Vec<u8>,                 // "\r\n" or "\n" (set after seeing first boundary line)
    nl_dash_boundary: Vec<u8>,   // nl + "--boundary"
    dash_boundary_dash: Vec<u8>, // "--boundary--"
    dash_boundary: Vec<u8>,      // "--boundary"
}

impl MultiPart {
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
            is_end: false,
            parts_read: 0,
            nl: (&b[..2]).to_vec(),
            nl_dash_boundary: (&b[..boundary.len() + 2]).to_vec(),
            dash_boundary_dash: (&b[2..]).to_vec(),
            dash_boundary: (&b[2..b.len() - 2]).to_vec(),
        }
    }

    pub fn next(&mut self, reader: &mut BufReader<TcpStream>) -> Option<&Part> {
        if self.is_end {
            return None;
        }

        let v = self.remaining_bytes.splitn(3, |&c| c == b'\n');

        let mut d = (&self.remaining_bytes[..]).to_vec();
        let mut buf = vec![0; 1024];
        if v.count() < 3 {
            println!("next2 read new");
            println!(
                "total: {}, current: {}",
                self.content_length, self.bytes_read
            );
            if self.bytes_read >= self.content_length {
                println!("next: previour full read, but end");
                return None;
            }
            match reader.read(&mut buf) {
                Ok(n) => {
                    if n <= 0 {
                        println!("next part error: n <= 0");
                        return None;
                    }
                    d.extend_from_slice(&buf[..n]);
                    self.bytes_read += n;
                }
                Err(err) => {
                    println!("next part error: {}", err);
                    return None;
                }
            }
            self.remaining_bytes = Vec::new();
        }

        if !d.contains(&b'\n') {
            println!("next no \\n");
            return None;
        }

        let splits = d.splitn(3, |&c| c == b'\n');
        for (i, line) in splits.into_iter().enumerate() {
            match i {
                // -----------------------------974767299852498929531610575
                0 => {
                    let mut line_boundary = line.to_vec();
                    if line_boundary.ends_with(b"\r") {
                        line_boundary.pop();
                    }
                    if line_boundary == self.dash_boundary {
                        continue;
                    }
                    if line_boundary == self.dash_boundary_dash {
                        println!("next no more part, end");
                        return None;
                    }
                }

                // Content-Disposition: form-data; name="nickname"
                // or
                // Content-Disposition: form-data; name="myFile"; filename="foo.txt"
                1 => {
                    let mut line_disposition = line.to_vec();
                    if !line.starts_with(b"Content-Disposition") {
                        println!("next Content-Disposition error");
                        return None;
                    }
                    if line_disposition.ends_with(b"\r") {
                        line_disposition.pop();
                    }
                    self.handle_content_disposition(&line_disposition);
                }

                // The Content-Type element only occures when part is file
                // Content-Type: text/plain\r\n
                // \r\n
                2 => {
                    if !line.starts_with(b"Content-Type") {
                        // \r
                        self.current_part.content_type = ContentType::None;
                        self.remaining_bytes = (&line[2..]).to_vec(); // no '\r\n'
                        break;
                    }

                    for (j, line) in line.splitn(2, |&c| c == b'\n').into_iter().enumerate() {
                        // \r
                        if j > 0 {
                            self.remaining_bytes = (&line[2..]).to_vec(); // no '\r\n'
                            break;
                        }

                        // Content-Type
                        let mut line_type = line.to_vec();
                        if line_type.ends_with(b"\r") {
                            line_type.pop();
                        }
                        let content_type: Vec<&str> = str::from_utf8(&line_type)
                            .unwrap()
                            .split(":")
                            .into_iter()
                            .collect();
                        self.current_part.content_type = ContentType::parse(content_type[1].trim());
                    }
                }
                _ => (),
            }
        }

        self.parts_read += 1;

        Some(&self.current_part)
    }

    pub fn body(&mut self, bufreader: &mut BufReader<TcpStream>) -> Result<Vec<u8>> {
        let size = 8192;
        let mut buf = vec![0; size];
        let mut result = Vec::new();

        // next read and match
        let mut out_doubt_index = usize::MAX;
        let mut out_dnd = 0;

        let len_nd = self.nl_dash_boundary.len();

        loop {
            let mut n = 0;
            let data = if self.remaining_bytes.len() > 0 {
                n = self.remaining_bytes.len();
                &self.remaining_bytes[..n]
            } else {
                if self.bytes_read >= self.content_length {
                    return Ok(vec![]);
                }

                match bufreader.read(&mut buf) {
                    Ok(bytes) => {
                        if bytes <= 0 {
                            println!("reach end when read body");
                            return Ok(result);
                        }
                        n = bytes;
                        self.bytes_read += n;
                        &buf[..n]
                    }

                    Err(err) => {
                        println!("Err occured when read body: {}", err);
                        return Err(err);
                    }
                }
            };

            if out_dnd > 0 && n >= out_dnd {
                let head = &result[out_doubt_index..]; // last remaining
                let tail = &data[..out_dnd]; // now need
                let mut full = Vec::new();
                full.extend_from_slice(head);
                full.extend_from_slice(tail);

                // found new boundary
                if self.nl_dash_boundary == &full[..] {
                    //remove previous data
                    let mut v = Vec::new();
                    v.extend_from_slice(&full[2..]);
                    v.extend_from_slice(&data[out_dnd..]);
                    self.remaining_bytes = v;

                    // println!(
                    //     "Matched, next head s1: {}",
                    //     str::from_utf8(&self.remaining_bytes[..]).unwrap()
                    // );
                    // println!("Matched, next head b1: {:?}", &self.remaining_bytes[..]);

                    for _ in 0..len_nd - out_dnd {
                        result.pop();
                    }

                    return Ok(result);
                }
            }

            // out_dnd = 0;

            let mut matched_index = size;
            let mut doubt_index = size;
            let mut dnd = 0;

            // find dash_boundary_dash
            for (i, c) in data.iter().enumerate() {
                if c != &b'\r' {
                    continue;
                }

                let mut should_find = false;
                should_find = if i + 1 < n {
                    data[i + 1] == b'\n'
                } else {
                    dnd = len_nd - 1;
                    false
                };

                if !should_find {
                    continue;
                };

                if i + 2 < n {
                    if data[i + 2] == b'-' {
                        if i + len_nd - 1 < n {
                            // found new boundary
                            if &data[i..i + len_nd] == self.nl_dash_boundary {
                                matched_index = i;
                                break;
                            }
                        } else {
                            dnd = i + len_nd - n; // need more
                        }
                    }
                } else {
                    dnd = len_nd - 2;
                }

                if dnd > 0 {
                    doubt_index = i;
                }
            }

            if matched_index != size {
                result.extend_from_slice(&data[..matched_index]);
                // store remaining bytes
                self.remaining_bytes = (&data[matched_index + 2..]).to_vec();

                if self.remaining_bytes == self.dash_boundary_dash {
                    self.is_end = true;
                }

                // println!(
                //     "Matched, next head s2: {}",
                //     str::from_utf8(&self.remaining_bytes[..]).unwrap()
                // );
                // println!("Matched, next head b2: {:?}", &self.remaining_bytes[..]);
                return Ok(result);
            } else {
                result.extend_from_slice(data);
            }

            self.bytes_read += n;
            if self.bytes_read >= self.content_length {
                return Ok(result);
            }

            // cache doubt
            if dnd > 0 {
                out_dnd = dnd;
                out_doubt_index = result.len() - (len_nd - dnd);
                // println!("dnd = {dnd}");
                // println!("dout_index = {doubt_index}");
                // println!("out_doubt_index = {out_doubt_index}");
            }

            self.remaining_bytes = Vec::new();
        }
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

    fn handle_content_disposition(&mut self, content_disposition: &[u8]) {
        let str = String::from_utf8(content_disposition.to_vec()).unwrap();
        let v: Vec<&str> = str.split(";").collect();
        if v.len() >= 2 {
            self.current_part.disposition = v[1].trim().to_string();
        } else {
            self.current_part.disposition = "".to_string();
        }
    }
}

// impl Iterator for Reader {
//     type Item = Part;
//
// }

type MIME_Header = HashMap<String, Vec<String>>;
