use std::{
    collections::HashMap,
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
};

use thread_pool::ThreadPool;
use context::{Context, ResponseFunc};

pub mod context;
mod request;
pub mod response;
mod thread_pool;

pub struct RustWeb {
    address: String,
    port: u32,
    map: Arc<Mutex<HashMap<String, fn(Context)>>>,
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
        self.map
            .as_ref()
            .lock()
            .unwrap()
            .insert(path.to_string(), handle_func);
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

fn handle_connection(map: Arc<Mutex<HashMap<String, fn(Context)>>>, stream: TcpStream) {
    let mut context = Context::new(stream);
    let key = &context.request.path[..];

    if let Some(f) = map.lock().unwrap().get(key).copied(){
        f(context);
    }else {
        context.error();
    }
}