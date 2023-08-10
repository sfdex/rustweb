use context::{Context, ContextFn};
use std::net::{TcpListener, TcpStream};
use thread_pool::ThreadPool;

mod content_type;
pub mod context;
mod request;
pub mod response;
mod router;
mod thread_pool;

pub struct RustWeb {
    address: String,
    port: u32,
}

pub fn build_server(address: &str, port: u32) -> RustWeb {
    RustWeb {
        address: address.to_string(),
        port,
    }
}

impl RustWeb {
    pub fn get(&self, path: &str, handle_func: fn(Context)) {
        let item = router::RoutingItem {
            path: path.to_string(),
            method: "GET".to_string(),
            func: handle_func,
        };

        router::insert(path, item);
    }

    pub fn post(&self, path: &str, handle_func: fn(Context)) {
        let item = router::RoutingItem {
            path: path.to_string(),
            method: "POST".to_string(),
            func: handle_func,
        };
        router::insert(path, item);
    }

    pub fn run(&self) {
        let pool = ThreadPool::new(4);
        let listener = TcpListener::bind(format!("{}:{}", self.address, self.port)).unwrap();

        for stream in listener.incoming() {
            let stream = stream.unwrap();

            pool.excute(move || {
                handle_connection(stream);
            });
        }
    }
}

fn handle_connection(stream: TcpStream) {
    match Context::new(stream) {
        Err(err) => {
            println!("error occured: {}", err);
        }

        Ok(mut context) => {
            let key = &context.request.path[..];

            if let Some(mapping) = router::find(key) {
                if context.request.method != mapping.method {
                    context.error_with_status(403, String::from("Method Not Allowed"));
                    return;
                }
                let f = mapping.func;
                f(context);
            } else {
                context.error();
            }
        }
    }
}
