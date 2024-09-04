use actix_web::{dev::Server, web, App, HttpServer};
use sqlx::PgPool;
use std::net::TcpListener;

use crate::routes::{health_check, subscribe};

pub fn run(tcp_socket: TcpListener, db_connection_pool: PgPool) -> Result<Server, std::io::Error> {
    let db_connection_pool = web::Data::new(db_connection_pool);
    let server = HttpServer::new(move || {
        App::new()
            .app_data(db_connection_pool.clone())
            .service(health_check)
            .service(subscribe)
    })
    .listen(tcp_socket)?
    .run();

    Ok(server)
}
