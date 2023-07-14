#![feature(trait_alias)]

mod http;
mod server;

pub use http::{Method, Request, Response};
pub use server::{Server, Handler};
