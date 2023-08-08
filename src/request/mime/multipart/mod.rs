use std::collections::HashMap;

pub mod part;

struct Form(HashMap<String, String>);
struct MultiPart {
    boundary:String,
    length:usize,
    form: Form,
    file: Form,
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
impl MultiPart {
    fn new() -> Self {
        Self {
            boundary:"".to_string(),
            length:0,
            form: Form(HashMap::new()),
            file: Form(HashMap::new()),
        }
    }

    fn init(&mut self, content_type:&str){
        let v:Vec<&str> = content_type.split("; ").collect();
    }
}
