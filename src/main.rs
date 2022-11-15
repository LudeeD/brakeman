#![deny(warnings)]
use axum::{response::Html, routing::get, Router};
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    // build our application with a route
    let app = Router::new().route("/plumbing/health", get(health_check));

    // run it
    let addr = SocketAddr::from(([127, 0, 0, 1], 7331));
    println!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn health_check() -> Html<&'static str> {
    Html("<h1>Hello, World!</h1>")
}
