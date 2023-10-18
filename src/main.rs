use server::{Method, Request, Response, Server};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut server = Server::new("127.0.0.1:3000");
    server.set_handler(Method::Get, "*", |request: Request| {
        Response::new(
            200,
            format!(
                "{} from {} with body:\n{}",
                request.method,
                request.endpoint,
                request.body.unwrap_or_default()
            ),
        )
        .with_header("Zggff", "12")
    });
    server.set_handler(Method::Get, "/", |_request: Request| {
        Response::new(200, "Hello There")
    });

    server.listen()?;
    Ok(())
}
