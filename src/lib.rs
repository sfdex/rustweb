use std::{
    collections::HashMap,
    io::Write,
    net::TcpStream,
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
}

pub fn build_server(address: &str, port: u32) -> RustWeb {
    RustWeb {
        address: address.to_string(),
        port,
    }
}

impl RustWeb {
    pub fn get(&self, path: &str, handle_func: fn(Context)) {
        println!("Hello");
    }
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

    fn set_header(&mut self,header:HashMap<String,String>){
        self.header = header
    }

    fn add_header(&mut self,key:&str,value:&str){
        self.header.insert(key.to_string(), value.to_string());
    }

    fn build() -> Response{
        Response{
            status_code: 0,
            status_message: String::from("OK"),
            header: HashMap::new(),
        }
    }

    fn get_status_line(&self) -> String{
        format!("HTTP/1.1 {} {}",self.status_code,self.status_message)
    }

    fn get_header(&mut self) -> String{
        let mut headers = String::new();
        for(key,value) in &self.header{
            headers.push_str(&key);
            headers.push_str(": ");
            headers.push_str(&value);
            headers.push_str("\n");
        }
        headers
    }
}

pub trait ResponseFunc {
    fn json(&mut self, by: &[u8]);
}

impl Context {
    pub fn get_header(&self, key: &str) -> String {
        self.header[key].clone()
    }
}

impl ResponseFunc for Context {
    fn json(&mut self, by: &[u8]) {
        let mut response = Response::build();
        response.add_header("Content-Length", &format!("{}",by.len()));
        response.add_header("Content-Type", "application/json");

        let response = format!(
            "{}\r\n\r\n
            {}\r\n
            ",response.get_status_line(),response.get_header()
        );

        // let sum = response.as_bytes() + by;
        self.stream.write_all(by).unwrap();
    }
}

pub fn usage_main() {
    let web = build_server("127.0.0.1", 7878);
    // web.get("hello",hello_handler);
}

fn hello_handler(mut c: Context) {
    let host = c.get_header("host");
    println!("HOST: {host}");
    let response = "{\"code\":200,\"message\":\"\"}";
    c.json(response.as_bytes());
}
