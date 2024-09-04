use rstest::*;
use rust_zero2prod::configuration;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::net::TcpListener;
use uuid::Uuid;

#[actix_web::test]
async fn test_health_endpoint() {
    let app = spawn_app().await;

    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/health", &app.server_address))
        .send()
        .await
        .expect("Failed to execute request");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[actix_web::test]
async fn test_valid_form_returns_200() {
    let app = spawn_app().await;

    let client = reqwest::Client::new();

    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = client
        .post(format!("{}/subscriptions", &app.server_address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status().as_u16(), 200);

    let database_subscriptor = sqlx::query!("SELECT email, name FROM subscriptions",)
        .fetch_one(&app.db_connection_pool)
        .await
        .expect("Failed to fetch saved subscription");

    assert_eq!(database_subscriptor.email, "ursula_le_guin@gmail.com");
    assert_eq!(database_subscriptor.name, "le guin");
}

#[rstest]
#[case("", "missing name and email")]
#[case("name=le%20guin", "missing email")]
#[case("email=ursula_le_guin%40gmail.com", "missing name")]
#[actix_web::test]
async fn test_invalid_form_returns_400(#[case] body: String, #[case] error: String) {
    let app = spawn_app().await;

    let client = reqwest::Client::new();

    let response = client
        .post(format!("{}/subscriptions", &app.server_address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body.clone())
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(
        response.status().as_u16(),
        400,
        "The API did not fail with 400 error when the payload was {}",
        error
    );
}

pub struct TestingApp {
    pub server_address: String,
    pub db_connection_pool: PgPool,
}

async fn spawn_app() -> TestingApp {
    let tcp_socket = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let binded_port = tcp_socket.local_addr().unwrap().port();
    let server_address = format!("http://127.0.0.1:{}", binded_port);

    let mut configuration =
        configuration::get_configuration().expect("Failed to read configuration");
    configuration.database.name = Uuid::new_v4().to_string();

    let db_connection_pool = create_testing_database(&configuration.database).await;

    let server = rust_zero2prod::startup::run(tcp_socket, db_connection_pool.clone())
        .expect("Failed to bind address");

    let _ = tokio::spawn(server);

    TestingApp {
        server_address,
        db_connection_pool,
    }
}

async fn create_testing_database(db_configuration: &configuration::DatabaseSettings) -> PgPool {
    let mut db_connection =
        PgConnection::connect(&db_configuration.get_connection_string_without_db())
            .await
            .expect("Failed to connect to Postgres.");

    db_connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, db_configuration.name).as_str())
        .await
        .expect("Failed to create testing database");

    let db_connection_pool = PgPool::connect(&db_configuration.get_connection_string())
        .await
        .expect("Failed to connect to Postgres database");

    sqlx::migrate!("./migrations")
        .run(&db_connection_pool)
        .await
        .expect("Failed to migrate database");

    db_connection_pool
}
