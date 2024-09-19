use rstest::*;
use wiremock::{
    matchers::{any, method, path},
    Mock, ResponseTemplate,
};

use crate::helpers::{spawn_app, ConfirmationLinks, TestingApp};

async fn create_unconfirmed_subscriber(app: &TestingApp) -> ConfirmationLinks {
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    let _mock_guard = Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .named("Create unconfirmed subscriber")
        .expect(1)
        .mount_as_scoped(&app.mock_email_server)
        .await;

    app.send_subscription_request(body.into())
        .await
        .error_for_status()
        .unwrap();

    let mock_email_requests = &app
        .mock_email_server
        .received_requests()
        .await
        .unwrap()
        .pop()
        .unwrap();
    app.get_email_confirmation_links(mock_email_requests)
}

async fn create_confirmed_subscriber(app: &TestingApp) {
    let confirmation_links = create_unconfirmed_subscriber(app).await;
    reqwest::get(confirmation_links.html_link)
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
}

#[actix_web::test]
async fn test_newsletters_are_not_delivered_to_unconfirmed_subscribers() {
    let app = spawn_app().await;
    create_unconfirmed_subscriber(&app).await;

    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&app.mock_email_server)
        .await;

    let newsletter_body = serde_json::json!({
        "title": "Newsletter title",
        "content": {
            "plain_text": "Newsletter body",
            "html": "<p>Newsletter body</p>"
        }
    });

    let response = app.send_newsletter(newsletter_body).await;

    assert_eq!(response.status().as_u16(), 200);
}

#[actix_web::test]
async fn test_newsletters_are_delivered_to_confirmed_subscribers() {
    let app = spawn_app().await;
    create_confirmed_subscriber(&app).await;

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.mock_email_server)
        .await;

    let newsletter_body = serde_json::json!({
        "title": "Newsletter title",
        "content": {
            "plain_text": "Newsletter body",
            "html": "<p>Newsletter body</p>"
        }
    });

    let response = app.send_newsletter(newsletter_body).await;

    assert_eq!(response.status().as_u16(), 200);
}

#[rstest]
#[case(serde_json::json!({"title": "pepe"}), "missing content")]
#[case(serde_json::json!({"content": {"text": "text", "html": "html"}}), "missing title")]
#[case(serde_json::json!({"title":"title", "content": {"text": "text"}}), "missing html content")]
#[case(serde_json::json!({"title":"title", "content": {"html": "html"}}), "missing text content")]
#[actix_web::test]
async fn test_newsletter_endpoint_returns_400_for_invalid_email_data(
    #[case] email_body: serde_json::Value,
    #[case] error: String,
) {
    let app = spawn_app().await;

    let response = app.send_newsletter(email_body).await;

    assert_eq!(
        response.status().as_u16(),
        400,
        "The API did not fail with 400 error when the payload was {}",
        error
    );
}
