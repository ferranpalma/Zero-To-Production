use rust_zero2prod::startup;
use std::net::TcpListener;

#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    let tcp_socket = TcpListener::bind("127.0.0.1:8000").expect("Failed to bind to port 8000");

    startup::run(tcp_socket)?.await
}
