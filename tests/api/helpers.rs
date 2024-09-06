use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::net::TcpListener;
use uuid::Uuid;

use rust_zero2prod::{
    configuration::{self, DatabaseSettings},
    email_client::EmailClient,
    telemetry,
};

// The tracing stack is only initialised once
static TRACING: Lazy<()> = Lazy::new(|| {
    let tracing_subscriber_default_filter_level = "debug".into();
    let tracing_subscriber_name = "test".into();

    if std::env::var("TEST_LOG").is_ok() {
        let tracing_subscriber = telemetry::get_tracing_subscriber(
            tracing_subscriber_name,
            tracing_subscriber_default_filter_level,
            std::io::stdout,
        );
        telemetry::init_tracing_subscriber(tracing_subscriber);
    } else {
        let tracing_subscriber = telemetry::get_tracing_subscriber(
            tracing_subscriber_name,
            tracing_subscriber_default_filter_level,
            std::io::sink,
        );
        telemetry::init_tracing_subscriber(tracing_subscriber);
    }
});

pub struct TestingApp {
    pub server_address: String,
    pub db_connection_pool: PgPool,
}

pub async fn spawn_app() -> TestingApp {
    Lazy::force(&TRACING);

    let tcp_socket = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let binded_port = tcp_socket.local_addr().unwrap().port();
    let server_address = format!("http://127.0.0.1:{}", binded_port);

    let mut configuration =
        configuration::get_configuration().expect("Failed to read configuration");
    configuration.database.name = Uuid::new_v4().to_string();

    let db_connection_pool = create_testing_database(&configuration.database).await;

    let timeout = configuration.email_client.get_timeout();
    let email_client = EmailClient::new(
        configuration.email_client.base_url,
        configuration.email_client.sender_email,
        configuration.email_client.api_token,
        timeout,
    );

    let server = rust_zero2prod::startup::run(tcp_socket, db_connection_pool.clone(), email_client)
        .expect("Failed to bind address");

    let _ = tokio::spawn(server);

    TestingApp {
        server_address,
        db_connection_pool,
    }
}

async fn create_testing_database(db_configuration: &DatabaseSettings) -> PgPool {
    let mut db_connection =
        PgConnection::connect_with(&db_configuration.get_testing_connect_options())
            .await
            .expect("Failed to connect to Postgres.");

    db_connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, db_configuration.name).as_str())
        .await
        .expect("Failed to create testing database");

    let db_connection_pool = PgPool::connect_with(db_configuration.get_connect_options())
        .await
        .expect("Failed to connect to Postgres database");

    sqlx::migrate!("./migrations")
        .run(&db_connection_pool)
        .await
        .expect("Failed to migrate database");

    db_connection_pool
}
