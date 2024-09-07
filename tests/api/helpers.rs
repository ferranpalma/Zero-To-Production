use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;
use wiremock::MockServer;

use rust_zero2prod::{
    configuration::{self, DatabaseSettings},
    startup::{get_db_connection_pool, Application},
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
    pub mock_email_server: MockServer,
}

impl TestingApp {
    pub async fn send_subscription_request(&self, body: String) -> reqwest::Response {
        reqwest::Client::new()
            .post(&format!("{}/subscriptions", &self.server_address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request")
    }
}

pub async fn spawn_app() -> TestingApp {
    Lazy::force(&TRACING);

    let mock_email_server = MockServer::start().await;

    // Randomize configuration values so all tests are executed in isolation
    let mut configuration =
        configuration::get_configuration().expect("Failed to read configuration");
    configuration.database.name = Uuid::new_v4().to_string();
    configuration.application.port = 0;
    configuration.email_client.base_url = mock_email_server.uri();

    create_testing_database(&configuration.database).await;

    let application = Application::build_application(&configuration)
        .await
        .expect("Failed to build application.");
    let server_address = format!("http://127.0.0.1:{}", application.get_port());

    let _ = tokio::spawn(application.run_server());

    TestingApp {
        db_connection_pool: get_db_connection_pool(&configuration.database),
        server_address,
        mock_email_server,
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
