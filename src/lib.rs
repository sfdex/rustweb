use std::{
    collections::HashMap,
    fs,
    io::{prelude::*, BufReader, Write},
    net::{TcpListener, TcpStream},
    str::Bytes,
    sync::{mpsc, Arc, Mutex},
    thread,
};
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }
        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }

    pub fn excute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.sender.as_ref().unwrap().send(job).unwrap();
    }

    pub fn excute2<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());

        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv();
            match message {
                Ok(job) => {
                    println!("Worker {id} got a job; executing.");

                    job();
                }
                Err(_) => {
                    println!("Worker {id} disconnected; shutting down.");
                    break;
                }
            }
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}

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
        println!("Hello");
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
    let context = Context::new(stream);
    let key = &context.path[..];
    println!("path = {}", key);

    let f = map.lock().unwrap().get(key).copied().unwrap();
    f(context);
}

pub struct Context {
    path: String,
    header: HashMap<String, String>,
    query: HashMap<String, String>,
    stream: TcpStream,
}

struct Response {
    status_code: u8,
    status_message: String,

    header: HashMap<String, String>,
    // body: [u8],
}

impl Response {
    fn set_status(&mut self, status_code: u8, status_message: String) {
        self.status_code = status_code;
        self.status_message = status_message;
    }

    fn set_header(&mut self, header: HashMap<String, String>) {
        self.header = header
    }

    fn add_header(&mut self, key: &str, value: &str) {
        self.header.insert(key.to_string(), value.to_string());
    }

    fn build() -> Response {
        Response {
            status_code: 0,
            status_message: String::from("OK"),
            header: HashMap::new(),
        }
    }

    fn get_status_line(&self) -> String {
        format!("HTTP/1.1 {} {}", self.status_code, self.status_message)
    }

    fn get_header(&mut self) -> String {
        let len = self.header.len();
        let mut count = 0;

        let mut headers = String::new();
        for (key, value) in &self.header {
            headers.push_str(&format!("{key}: {value}"));
            if count < len - 1 {
                headers.push_str("\r\n");
            }
            count += 1;
        }

        headers
    }
}

pub trait ResponseFunc {
    fn json(&mut self, content: &str);
}

impl Context {
    pub fn get_header(&self, key: &str) -> String {
        self.header
            .get(key)
            .unwrap_or(&String::from(""))
            .to_string()
    }

    fn new(stream: TcpStream) -> Context {
        let buf_reader = BufReader::new(&stream);
        let http_request: Vec<String> = buf_reader
            .lines()
            .map(|result| result.unwrap())
            .take_while(|line| !line.is_empty())
            .collect();

        let mut proto_version = "";
        let mut path = "";
        let mut method = "";
        let mut request_header = HashMap::new();

        for (index, line) in http_request.iter().enumerate() {
            if index == 0 {
                for (index, text) in line.split_whitespace().enumerate() {
                    match index {
                        0 => method = text,
                        1 => path = text,
                        _ => proto_version = text,
                    }
                }
            } else {
                let mut iter = line.split(": ");
                request_header.insert(
                    iter.next().unwrap().to_string(),
                    iter.next().unwrap().to_string(),
                );
            }
        }
        Context {
            path: path.to_string(),
            header: request_header,
            query: HashMap::new(),
            stream,
        }
    }
}

impl ResponseFunc for Context {
    fn json(&mut self, content: &str) {
        
        let mut response = Response::build();

        response.add_header("Content-Type", "application/json");
        response.add_header("Content-Length", &format!("{}", content.len()));

        let response = format!(
            "{}\r\n{}\r\n\r\n{}",
            response.get_status_line(),
            response.get_header(),
            content
        );
        
        self.stream.write_all(response.as_bytes()).unwrap();
    }
}