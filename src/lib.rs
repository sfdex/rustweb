use std::{
    collections::HashMap,
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
};

use context::{Context, ContextFn};
use thread_pool::ThreadPool;

mod content_type;
pub mod context;
mod request;
pub mod response;
mod thread_pool;

pub struct RustWeb {
    address: String,
    port: u32,
    map: Arc<Mutex<HashMap<String, RequestMapping>>>,
}

pub struct RequestMapping {
    path: String,
    method: String,
    pub func: fn(Context),
}

pub fn build_server(address: &str, port: u32) -> RustWeb {
    RustWeb {
        address: address.to_string(),
        port,
        map: Arc::new(Mutex::new(HashMap::new())),
    }
}

impl RustWeb {
    pub fn get(&self, path: &str, handle_func: fn(Context)) {
        self.map.as_ref().lock().unwrap().insert(
            path.to_string(),
            RequestMapping {
                path: path.to_string(),
                method: "GET".to_string(),
                func: handle_func,
            },
        );
    }

    pub fn post(&self, path: &str, handle_func: fn(Context)) {
        self.map.as_ref().lock().unwrap().insert(
            path.to_string(),
            RequestMapping {
                path: path.to_string(),
                method: "POST".to_string(),
                func: handle_func,
            },
        );
    }

    pub fn run(&self) {
        let pool = ThreadPool::new(4);
        let listener = TcpListener::bind(format!("{}:{}", self.address, self.port)).unwrap();

        for stream in listener.incoming() {
            let stream = stream.unwrap();
            let map = Arc::clone(&self.map);

            pool.excute(move || {
                handle_connection(map, stream);
            });
        }
    }
}

fn handle_connection(map: Arc<Mutex<HashMap<String, RequestMapping>>>, stream: TcpStream) {
    match Context::new(stream) {
        Err(err) => {
            println!("error occured: {}", err);
        }

        Ok(mut context) => {
            let key = &context.request.path[..];

            if let Some(mapping) = map.lock().unwrap().get(key) {
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
