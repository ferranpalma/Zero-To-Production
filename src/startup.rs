use actix_web::{dev::Server, web, App, HttpServer};
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;

use crate::{
    configuration::Settings,
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

pub async fn build_server(configuration: Settings) -> Result<Server, std::io::Error> {
    let db_connection_pool =
        PgPoolOptions::new().connect_lazy_with(configuration.database.get_connect_options());

    let email_client_timeout = configuration.email_client.get_timeout();
    let email_client = EmailClient::new(
        configuration.email_client.base_url,
        configuration.email_client.sender_email,
        configuration.email_client.api_token,
        email_client_timeout,
    );

    let server_address = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );
    let server_tcp_socket = TcpListener::bind(server_address)?;

    run(server_tcp_socket, db_connection_pool, email_client)
}
