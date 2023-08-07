use rustweb::context::{Context, ContextFn};

fn main() {
    let web = rustweb::build_server("127.0.0.1", 7878);
    web.get("/hello", hello_handler);
    web.post("/update", update_handler);
    web.run();
}

fn hello_handler(mut c: Context) {
    let content = "{\"code\":200,\"message\":\"\"}";
    c.json(content);
}

fn update_handler(mut c: Context) {
    let content = "{\"code\":200,\"message\":\"\"}";
    // let body = c.request.body();
    // println!("\n{}", String::from_utf8(body).unwrap());
    // println!("body: len = {}",body.len());
    c.request.parse_post_form();
    println!("form: {:?}", c.request.post_form);
    c.json(content);
}
