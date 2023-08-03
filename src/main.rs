use rustweb::context::Context;
use rustweb::context::ResponseFunc;
fn main() {
    let web = rustweb::build_server("127.0.0.1", 7878);
    web.get("/hello", hello_handler);
    web.post("/update", update_handler);
    web.run();
}

fn hello_handler(mut c: Context) {
    let host = c.get_header("Host");
    // println!("HOST: {host}");
    let content = "{\"code\":200,\"message\":\"\"}";
    c.json(content);
}

fn update_handler(mut c:Context){
    let content = "{\"code\":200,\"message\":\"\"}";
    // let body = c.request.body();
    // println!("body: {:?}",body);
    c.json(content);
}