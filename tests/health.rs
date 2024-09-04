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

fn spawn_app() -> String {
    let tcp_socket = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let binded_port = tcp_socket.local_addr().unwrap().port();

    let server = rust_zero2prod::run(tcp_socket).expect("Failed to bind address");
    let _ = tokio::spawn(server);

    format!("http://127.0.0.1:{}", binded_port)
}
