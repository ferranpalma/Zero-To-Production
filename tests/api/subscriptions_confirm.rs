use reqwest::Url;
use wiremock::{
    matchers::{method, path},
    Mock, ResponseTemplate,
};

use crate::helpers::spawn_app;

#[actix_web::test]
async fn test_link_returned_by_subscribe_endpoint_returns_200() {
    let app = spawn_app().await;

    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.mock_email_server)
        .await;

    app.send_subscription_request(body.into()).await;

    let email_server_first_request = &app
        .mock_email_server
        .received_requests()
        .await
        .expect("No received requests in the email server")
        .first()
        .cloned()
        .expect("Unable to extract first email server request");
    let request_body: serde_json::Value = serde_json::from_slice(&email_server_first_request.body)
        .expect("Failed to get request body");

    let get_link = |s: &str| {
        let links: Vec<_> = linkify::LinkFinder::new()
            .links(s)
            .filter(|l| *l.kind() == linkify::LinkKind::Url)
            .collect();
        assert_eq!(links.len(), 1);
        links.first().unwrap().as_str().to_owned()
    };
    let raw_confirmation_link = &get_link(request_body["HtmlBody"].as_str().unwrap());
    let mut confirmation_link = Url::parse(raw_confirmation_link).unwrap();
    assert_eq!(confirmation_link.host_str().unwrap(), "127.0.0.1");
    confirmation_link.set_port(Some(app.server_port)).unwrap();

    let confirm_endpoint_response = reqwest::get(confirmation_link)
        .await
        .expect("Failed to hit confirmation endpoint");
    assert_eq!(confirm_endpoint_response.status().as_u16(), 200);
}

#[actix_web::test]
async fn test_confirmation_without_token_is_rejected() {
    let app = spawn_app().await;

    let response = reqwest::get(&format!("{}/subscriptions/confirm", app.server_address))
        .await
        .expect("Failed to hit confirm endpoint");

    assert_eq!(response.status().as_u16(), 400);
}
