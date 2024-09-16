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
    let confirmation_links = app.get_email_confirmation_links(email_server_first_request);

    let confirm_endpoint_response = reqwest::get(confirmation_links.html_link)
        .await
        .expect("Failed to hit subscription confirmation endpoint");

    assert_eq!(confirm_endpoint_response.status().as_u16(), 200);
}

#[actix_web::test]
async fn test_confirmation_link_hit_twice_works() {
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
    let confirmation_links = app.get_email_confirmation_links(email_server_first_request);

    let confirm_endpoint_response = reqwest::get(confirmation_links.html_link.clone())
        .await
        .expect("Failed to hit subscription confirmation endpoint");
    assert_eq!(confirm_endpoint_response.status().as_u16(), 200);

    let confirm_endpoint_response = reqwest::get(confirmation_links.html_link)
        .await
        .expect("Failed to hit subscription confirmation endpoint");
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

#[actix_web::test]
async fn test_confirmation_with_invalid_token_is_rejected() {
    let app = spawn_app().await;

    let response = reqwest::get(&format!(
        "{}/subscriptions/confirm?token=invalid_token",
        app.server_address
    ))
    .await
    .expect("Failed to hit confirm endpoint");

    assert_eq!(response.status().as_u16(), 400);
}

#[actix_web::test]
async fn test_confirmation_with_non_existent_token_is_rejected() {
    let app = spawn_app().await;

    let response = reqwest::get(&format!(
        "{}/subscriptions/confirm?subscription_token=abcdefgHIJK0123456789mncd",
        app.server_address
    ))
    .await
    .expect("Failed to hit confirm endpoint");

    assert_eq!(response.status().as_u16(), 401);
}

#[actix_web::test]
async fn test_confirmation_endpoint_confirms_subscriber_in_database() {
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
    let confirmation_links = app.get_email_confirmation_links(email_server_first_request);

    reqwest::get(confirmation_links.html_link)
        .await
        .expect("Failed to hit subscription confirmation endpoint");

    let database_subscriptor = sqlx::query!("SELECT email, name, status FROM subscriptions",)
        .fetch_one(&app.db_connection_pool)
        .await
        .expect("Failed to fetch subscription in the database");
    assert_eq!(database_subscriptor.email, "ursula_le_guin@gmail.com");
    assert_eq!(database_subscriptor.name, "le guin");
    assert_eq!(database_subscriptor.status, "confirmed");
}
