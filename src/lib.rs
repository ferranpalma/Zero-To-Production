use actix_web::{dev::Server, get, App, HttpResponse, HttpServer};
use std::net::TcpListener;

#[get("/health")]
async fn health() -> HttpResponse {
    HttpResponse::Ok().finish()
}

pub fn run(tcp_socket: TcpListener) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(|| App::new().service(health))
        .listen(tcp_socket)?
        .run();

    Ok(server)
}
