use rustweb::context::{Context, ContextFn};
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

fn main() {
    let web = rustweb::build_server("0.0.0.0", 7878);

    web.get("/hello", hello_handler);
    web.get("/net/ip", ip_handler);
    web.get("/file/apk", apk_handler);

    web.post("/update", update_handler);
    web.post("/file/upload", upload_handler);
    web.post("/file/multipart", multipart_handler);

    web.run();
}

fn hello_handler(mut c: Context) {
    let content = "{\"code\":200,\"message\":\"\"}";
    c.json(content.as_bytes());
}

fn update_handler(mut c: Context) {
    let content = "{\"code\":200,\"message\":\"\"}";
    // let body = c.request.body();
    // println!("\n{}", String::from_utf8(body).unwrap());
    // println!("body: len = {}",body.len());
    c.request.parse_post_form();
    println!("form: {:?}", c.request.post_form);
    c.json(content.as_bytes());
}

// http://127.0.0.1:7878/file/apk?brand=gwm&version=1609&filename=wp1609.apk
fn apk_handler(mut c: Context) {
    let parent_dir = "/Users/sfdex/File/apk";
    let brand = c.request.query("brand");
    let version = c.request.query("version");
    let filename = c.request.query("filename");
    if brand.is_empty() || filename.is_empty() {
        c.error();
        return;
    }
    
    /*let path = format!("{}/{}/{}", parent_dir, brand,
        if version.is_empty() {
            filename.to_string()
        } else {
            format!("{}/{}", version, filename)
        }
    );*/

    let mut path = PathBuf::from(parent_dir);
    path.push(brand);
    if !version.is_empty() { path.push(version)  }
    path.push(&filename);

    let file_result = File::open(&path);
    println!("apk_handler, path={:?}, {:?}", path, file_result);

    match file_result {
        Ok(file) => c.file(file, filename),
        Err(_) => c.error(),
    }
}

fn upload_handler(mut c: Context) {
    let mut buf = vec![0; 8192];
    let mut file = File::create("test.img").unwrap();

    let mut total = 0usize;
    let length: usize = c.request.header_first("Content-Length").parse().unwrap();

    loop {
        match c.request.read_body(&mut buf) {
            Ok(0) => {
                println!("End");
                break;
            }
            Ok(n) => {
                file.write_all(&buf[0..n]).unwrap();
                total += n;
                println!("length = {length}, total = {total}, n = {n}");
                if total >= length {
                    println!("last: {:?}", &buf[n - 300..n]);
                    println!("Finish");
                    break;
                }
            }
            Err(e) => {
                println!("upload_handler error: {}", e);
                break;
            }
        }
    }
    file.flush().unwrap();
    let metadata = file.metadata().unwrap();
    println!("Recv file size: {}", metadata.len());
    let content = "{\"code\":200,\"message\":\"Upload finish!\"}";
    c.json(content.as_bytes());
}

fn multipart_handler(mut c: Context) {
    let reader = c.request.multipart();

    loop {
        let part = reader.next();
        if let None = part {
            break;
        }
        let part = part.unwrap();
        println!("disposition: {}", part.disposition);
        println!("contentType: {:?}", part.content_type);

        let body = reader.part_body().unwrap();
        println!("body: {}", String::from_utf8(body.to_vec()).unwrap());
    }

    c.ok();
}

fn ip_handler(mut c: Context) {
    let x_real_ip = c.request.header_first("X-Real-IP");
    let ip = if !x_real_ip.is_empty() {
        x_real_ip
    } else {
        c.request.address.ip().to_string()
    };
    let content = format!("{{\"code\":200,\"ip\":\"{}\"}}", ip);
    c.json(content.as_bytes());
}
