# rustweb
## Usage
### init
```
fn main(){
    let web = rustweb::build_server(address, port);

    web.get("/hello", hello_handler);
    web.post("/update", update_handler);
    ...

    web.run();
}
```

### custom response
```
struct Response{
    code: u8,
    message: String,
    data: obj,
}
```

### handler
```
fn hello_handler(c: rustweb::Context){
    let header = c.header(); // map
    let token = c.get_header("token"); // string
    ...

    let param1 = c.query("param1");
    let param2 = c.query("param2");
    ...

    let response = Response{
        code: 200,
        message: String::from("Ok"),
        data: String::from("Hello GET!"),
    };

    c.add_header("{key}","{value}");
    ...
    
    c.json(response);
}

fn update_handler(c: rustweb::Context){
    let header = c.header(); // map
    let token = c.get_header("token"); // string
    ...

    let body = c.body();

    let response = Response{
        code: 200,
        message: String::from("Ok"),
        data: String::from("Hello POST!"),
    };

    c.add_header("{key}","{value}");
    ...

    c.json(response);
}
```