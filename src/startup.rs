use actix_web::{dev::Server, web, App, HttpServer};
use sqlx::PgPool;
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;

use crate::{
    email_client::EmailClient,
    routes::{health_check, subscribe},
};

pub fn run(
    tcp_socket: TcpListener,
    db_connection_pool: PgPool,
    email_client: EmailClient,
) -> Result<Server, std::io::Error> {
    let db_connection_pool = web::Data::new(db_connection_pool);
    let http_email_client = web::Data::new(email_client);
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .app_data(db_connection_pool.clone())
            .app_data(http_email_client.clone())
            .service(health_check)
            .service(subscribe)
    })
    .listen(tcp_socket)?
    .run();

    Ok(server)
}
