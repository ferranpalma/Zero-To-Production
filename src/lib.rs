use actix_web::{dev::Server, get, post, web, App, HttpResponse, HttpServer};
use serde::Deserialize;
use std::net::TcpListener;

#[get("/health")]
async fn health() -> HttpResponse {
    HttpResponse::Ok().finish()
}

#[derive(Deserialize)]
struct SubscriberData {
    email: String,
    name: String,
}

#[post("/subscriptions")]
async fn subscribe(_form: web::Form<SubscriberData>) -> HttpResponse {
    HttpResponse::Ok().finish()
}

pub fn run(tcp_socket: TcpListener) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(|| App::new().service(health).service(subscribe))
        .listen(tcp_socket)?
        .run();

    Ok(server)
}
