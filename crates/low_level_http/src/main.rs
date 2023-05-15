use http::header::HOST;
use http::{Request, Response};
use hyper::{server::conn::Http, service::service_fn, Body};
use std::{convert::Infallible, net::SocketAddr};
use tokio::net::TcpListener;
use tokio::net::TcpStream;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let addr: SocketAddr = ([127, 0, 0, 1], 3001).into();

    println!("Starting server {}...", addr);

    let tcp_listener = TcpListener::bind(addr).await?;
    loop {
        let (tcp_stream, socket_addr) = tcp_listener.accept().await?;
        println!("Socket address {}", socket_addr);
        tokio::task::spawn(async move {
            if let Err(http_err) = Http::new()
                .http1_only(true)
                .http1_keep_alive(true)
                .serve_connection(tcp_stream, service_fn(handler))
                .await
            {
                eprintln!("Error while serving HTTP connection: {}", http_err);
            }
        });
    }
}

async fn handler(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    match req.uri().path() {
        "/connect" => connect_handler(req).await,
        _ => unknown_handler(req),
    }
}

async fn connect_handler(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let (_parts, body) = req.into_parts();
    let body = hyper::body::to_bytes(body).await.unwrap();
    let target_address = String::from_utf8(body.to_vec()).unwrap();
    println!("Connecting to target address... {:?}", target_address);

    let target_stream = TcpStream::connect(target_address.clone()).await.unwrap();
    println!("Connection made! ✅");
    println!("Attempting TCP handshake...");
    let (mut request_sender, connection) =
        hyper::client::conn::handshake(target_stream).await.unwrap();

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Error in connection: {}", e);
        }
    });

    println!("Constructing request...");
    let connect_request = Request::builder()
        .uri("/")
        .header(HOST, "http://127.0.0.1:3000")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    println!("Sending request...");
    let response = request_sender.send_request(connect_request).await.unwrap();

    println!("Received response! {:?}", response);
    Ok(response)
}

fn unknown_handler(_req: Request<Body>) -> Result<Response<Body>, Infallible> {
    Ok(Response::new(Body::from("Unknown 404")))
}
