use std::{collections::HashMap, io::BufReader, net::TcpStream};

use crate::content_type::ContentType;

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
pub struct Part {
    header: MIME_Header,
    disposition: String,
    disposition_params: HashMap<String, String>,
    content_type: ContentType,
}

// Reader is an iterator over parts in a MIME multipart body.
// Reader's underlying parser consumes its input as needed. Seeking isn't supported.
struct Reader {
    current_part: Part,
    parts_read: usize,

    nl: Vec<u8>,                 // "\r\n" or "\n" (set after seeing first boundary line)
    nl_dash_boundary: Vec<u8>,   // nl + "--boundary"
    dash_boundary_dash: Vec<u8>, // "--boundary--"
    dash_boundary: Vec<u8>,      // "--boundary"
}

impl Reader {
    pub fn new(reader: &BufReader<TcpStream>, boundary: String) -> Reader {
        // b := []byte("\r\n--" + boundary + "--")
        // let b = b"\r\n--".to_vec().extend(boundary.as_bytes().to_vec()).extend((b"--").to_vec());
        panic!()
    }
}

type MIME_Header = HashMap<String, Vec<String>>;
