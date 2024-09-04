use rust_zero2prod::{configuration, startup};
use sqlx::PgPool;
use std::net::TcpListener;

#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    let configuration = configuration::get_configuration().expect("Failed to read configuration");

    let db_connection_pool = PgPool::connect(&configuration.database.get_connection_string())
        .await
        .expect("Failed to connect to Postgres");

    let address = format!("127.0.0.1:{}", configuration.port);
    let tcp_socket = TcpListener::bind(address)?;

    startup::run(tcp_socket, db_connection_pool)?.await
}
