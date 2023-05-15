use std::{convert::Infallible, net::SocketAddr};

use hyper::server::conn::AddrStream;
use hyper::server::Server;
use hyper::{
    service::{make_service_fn, service_fn},
    Body, Request, Response,
};

async fn handle(_req: Request<Body>) -> Result<Response<Body>, Infallible> {
    Ok(Response::new(Body::from("Hello there")))
}

#[tokio::main]
async fn main() {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    let make_service = make_service_fn(|socket: &AddrStream| {
        let remote_addr = socket.remote_addr();
        println!("addr {}", remote_addr);
        async move { Ok::<_, Infallible>(service_fn(handle)) }
    });

    let server = Server::bind(&addr).serve(make_service);

    println!("Server listing on {}...", addr);
    if let Err(e) = server.await {
        eprintln!("Server failed: {}", e);
    }
}
