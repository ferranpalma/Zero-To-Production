use rstest::*;

use super::helpers::spawn_app;

#[actix_web::test]
async fn test_valid_form_returns_201() {
    let app = spawn_app().await;

    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = app.send_subscription_request(body.into()).await;

    assert_eq!(response.status().as_u16(), 201);

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
async fn test_form_with_missing_data_returns_400(#[case] body: String, #[case] error: String) {
    let app = spawn_app().await;

    let response = app.send_subscription_request(body).await;

    assert_eq!(
        response.status().as_u16(),
        400,
        "The API did not fail with 400 error when the payload was {}",
        error
    );
}

#[rstest]
#[case("name=&email=", "empty name and email")]
#[case("name=&email=ursula_le_guin%40gmail.com", "empty name")]
#[case("name=Ursula&email=", "empty email")]
#[case("name=ursula&email=definitely-not-an-email", "empty name and email")]
#[actix_web::test]
async fn test_form_with_invalid_data_returns_400(#[case] body: String, #[case] error: String) {
    let app = spawn_app().await;

    let response = app.send_subscription_request(body).await;

    assert_eq!(
        response.status().as_u16(),
        400,
        "The API did not fail with 400 error when the payload was {}",
        error
    );
}
