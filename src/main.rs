use rustweb::{Context, ResponseFunc};

fn main() {
    let web = rustweb::build_server("127.0.0.1", 7878);
    web.get("/hello", hello_handler);
    web.run();
}

fn hello_handler(mut c: Context) {
    let host = c.get_header("Host");
    println!("HOST: {host}");
    let content = "{\"code\":200,\"message\":\"\"}";
    c.json(content);
}
