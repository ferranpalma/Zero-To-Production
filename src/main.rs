use sqlx::postgres::PgPoolOptions;
use std::net::TcpListener;

use rust_zero2prod::{configuration, startup, telemetry};

#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    let configuration = configuration::get_configuration().expect("Failed to read configuration");

    let tracing_subsriber =
        telemetry::get_tracing_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    telemetry::init_tracing_subscriber(tracing_subsriber);

    let db_connection_pool =
        PgPoolOptions::new().connect_lazy_with(configuration.database.get_connect_options());

    let address = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );
    let tcp_socket = TcpListener::bind(address)?;

    startup::run(tcp_socket, db_connection_pool)?.await?;
    Ok(())
}
