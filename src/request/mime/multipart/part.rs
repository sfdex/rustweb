use crate::content_type::ContentType;
use std::io::{prelude::*, BufReader};
use std::{collections::HashMap, net::TcpStream};

pub struct Part {
    header: MIME_Header,
    disposition: String,
    disposition_params: HashMap<String, String>,
    content_type: ContentType,
}

impl Part {
    fn new() -> Self {
        Self {
            header: MIME_Header::new(),
            disposition: "".to_string(),
            disposition_params: HashMap::new(),
            content_type: ContentType::None,
        }
    }
}

// Reader is an iterator over parts in a MIME multipart body.
// Reader's underlying parser consumes its input as needed. Seeking isn't supported.
struct Reader {
    reader: BufReader<TcpStream>,
    content_length: usize,
    current_part: Part,
    parts_read: usize,

    nl: Vec<u8>,                 // "\r\n" or "\n" (set after seeing first boundary line)
    boundary: Vec<u8>,           // boundary
    nl_boundary: Vec<u8>,        // nl + boundary
    nl_dash_boundary: Vec<u8>,   // nl + "--boundary"
    dash_boundary_dash: Vec<u8>, // "--boundary--"
    dash_boundary: Vec<u8>,      // "--boundary"
    boundary_dash: Vec<u8>,      // boundary--
}

impl Reader {
    pub fn new(reader: BufReader<TcpStream>, boundary: String, content_length: usize) -> Self {
        // b := []byte("\r\n--" + boundary + "--")
        let mut b = Vec::new();
        // b.extend_from_slice(b"\r\n--");

        b.extend_from_slice(b"\r\n");
        b.extend_from_slice(boundary.as_bytes());
        b.extend_from_slice(b"--");

        Self {
            reader,
            content_length,
            current_part: Part::new(),
            parts_read: 0,
            nl: (&b[..2]).to_vec(),
            boundary: boundary.as_bytes().to_vec(),
            nl_boundary: (&b[..boundary.len() + 2]).to_vec(),
            nl_dash_boundary: (&b[..boundary.len() + 2]).to_vec(),
            dash_boundary_dash: (&b[2..]).to_vec(),
            dash_boundary: (&b[2..b.len() - 2]).to_vec(),
            boundary_dash: (&b[2..]).to_vec(),
        }
    }
}

impl Iterator for Reader {
    type Item = Part;

    fn next(&mut self) -> Option<Self::Item> {
        let mut buf = vec![0; 4096];
        let reader = &mut self.reader;
        loop {
            match reader.read(&mut buf) {
                Ok(0) => {
                    println!("MultiPart end");
                    break;
                }
                Ok(n) => process_part(
                    &mut self.current_part,
                    &buf[0..n],
                    &self.nl,
                    &self.boundary,
                    &self.boundary_dash,
                ),
                Err(err) => {
                    println!("MultiPart error: {}", err)
                }
            }
        }

        // reader.
        None
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
fn process_part(
    part: &mut Part,
    buf: &[u8],
    nl: &Vec<u8>,
    boundary: &Vec<u8>,
    boundary_dash: &Vec<u8>,
) {
    let parts: Vec<&[u8]> = buf.split(|&b| b == b'\n').collect();

    let mut iter = parts.iter();
    if let Some(&line_boundary) = iter.next() {
        if line_boundary == boundary {
            if let Some(&line_content_disposition) = iter.next() {
                handle_content_disposition(part, line_content_disposition);
                if let Some(&line_content_type) = iter.next() {
                    part.content_type =
                        ContentType::parse(&String::from_utf8(line_content_type.to_vec()).unwrap())
                    // todo: process body
                }
            }
        }
    }
}

fn handle_content_disposition(part: &mut Part, content_disposition: &[u8]) {
    if content_disposition.starts_with(b"Content-Disposition") {
        let str = String::from_utf8(content_disposition.to_vec()).unwrap();
        let v: Vec<&str> = str.split(";").collect();
        if v.len() >= 2 {
            part.disposition = v[1].trim().to_string();
        }
        part.disposition = "".to_string();
    }
}

type MIME_Header = HashMap<String, Vec<String>>;
