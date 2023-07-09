use std::{
    collections::HashMap,
    error::Error,
    net::{TcpListener, TcpStream},
    sync::{Arc, OnceLock},
};

use http::{Method, Request, Response};

mod http;

type HandlerKey = (Method, String);
type Handler = Arc<dyn Fn(Request) -> Response + Send + Sync>;
static HANDLERS: OnceLock<HashMap<HandlerKey, Handler>> = OnceLock::new();

macro_rules! set_handler {
    ($handlers: ident, $method: expr, $endpoint: literal, $handler: expr) => {
        let handler: Handler = Arc::new($handler);
        $handlers.insert(($method, $endpoint.to_string()), handler);
    };
}

fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("127.0.0.1:3000")?;
    let mut handlers = HashMap::new();

    set_handler!(handlers, Method::Get, "*", |request: Request| {
        Response::new(
            200,
            format!(
                "{} from {} with body:\n{}",
                request.method,
                request.endpoint,
                request.body.unwrap_or(String::new())
            ),
        )
        .with_header("Zggff", "12")
    });
    set_handler!(handlers, Method::Get, "/", |_request: Request| {
        Response::new(200, format!("Hello There",))
    });

    if HANDLERS.set(handlers).is_err() {
        panic!("failed to initialise handlers")
    }

    for stream in listener.incoming().flatten() {
        handle_stream(stream)?;
    }
    Ok(())
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
