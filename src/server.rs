use std::{
    collections::HashMap,
    error::Error,
    net::{TcpListener, TcpStream, ToSocketAddrs},
    sync::{Arc, OnceLock},
};

use crate::http::{Method, Request, Response};

pub trait Handler = Fn(Request) -> Response + Send + Sync + 'static;
pub type Handlers = HashMap<(Method, String), Arc<dyn Handler>>;
pub static HANDLERS: OnceLock<Handlers> = OnceLock::new();

pub struct Server {
    listener: TcpListener,
    handlers: Handlers,
}

impl Server {
    pub fn new<A: ToSocketAddrs>(addr: A) -> Self {
        Self {
            listener: TcpListener::bind(addr).unwrap(),
            handlers: HashMap::new(),
        }
    }
    pub fn set_handler<S: Into<String>, F: Handler>(
        &mut self,
        method: Method,
        endpoint: S,
        handler: F,
    ) {
        let handler = Arc::new(handler);
        self.handlers.insert((method, endpoint.into()), handler);
    }
    pub fn listen(&mut self) -> Result<(), Box<dyn Error>> {
        if HANDLERS.set(self.handlers.clone()).is_err() {
            panic!("failed to initialise handlers")
        }

        for stream in self.listener.incoming().flatten() {
            handle_stream(stream)?;
        }
        Ok(())
    }
}

fn handle_stream(mut stream: TcpStream) -> Result<(), Box<dyn Error>> {
    let request = Request::read(&stream);
    if let Some(handler) = HANDLERS
        .get()
        .and_then(|h| h.get(&(request.method, request.endpoint.clone())))
    {
        let response = handler(request);
        response.write(&mut stream)?;
    } else if let Some(handler) = HANDLERS
        .get()
        .and_then(|h| h.get(&(request.method, "*".to_string())))
    {
        let response = handler(request);
        response.write(&mut stream)?;
    } else {
        let response = Response::new(404, "Not Found".to_string());
        response.write(&mut stream)?;
    }
    Ok(())
}
