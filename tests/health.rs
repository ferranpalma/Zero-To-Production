use rstest::*;
use std::net::TcpListener;

#[actix_web::test]
async fn test_health_endpoint() {
    let address = spawn_app();

    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/health", &address))
        .send()
        .await
        .expect("Failed to execute request");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[actix_web::test]
async fn test_valid_form_returns_200() {
    let address = spawn_app();

    let client = reqwest::Client::new();

    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = client
        .post(format!("{}/subscriptions", &address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status().as_u16(), 200);
}

#[rstest]
#[case("", "missing name and email")]
#[case("name=le%20guin", "missing email")]
#[case("email=ursula_le_guin%40gmail.com", "missing name")]
#[actix_web::test]
async fn test_invalid_form_returns_400(#[case] body: String, #[case] error: String) {
    let address = spawn_app();

    let client = reqwest::Client::new();

    let response = client
        .post(format!("{}/subscriptions", &address))
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

fn spawn_app() -> String {
    let tcp_socket = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let binded_port = tcp_socket.local_addr().unwrap().port();

    let server = rust_zero2prod::startup::run(tcp_socket).expect("Failed to bind address");
    let _ = tokio::spawn(server);

    format!("http://127.0.0.1:{}", binded_port)
}
