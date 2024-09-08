use actix_web::{post, web, HttpResponse};
use chrono::Utc;
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{email_client::EmailClient, models::NewSubscriber, startup::ApplicationBaseUrl};

#[derive(Deserialize)]
pub struct SubscriberData {
    pub email: String,
    pub name: String,
}

#[tracing::instrument(
    name = "Add a new subscriber",
    skip(subscriber_data, db_connection_pool, email_client, application_base_url),
    fields(
        subscriber_name = %subscriber_data.name,
        subscriber_email = %subscriber_data.email,
        )
)]
#[post("/subscriptions")]
pub async fn subscribe(
    subscriber_data: web::Form<SubscriberData>,
    db_connection_pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
    application_base_url: web::Data<ApplicationBaseUrl>,
) -> HttpResponse {
    let new_subscriber = match subscriber_data.0.try_into() {
        Ok(x) => x,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };

    if insert_subscriber_into_database(&db_connection_pool, &new_subscriber)
        .await
        .is_err()
    {
        return HttpResponse::InternalServerError().finish();
    };

    if send_confirmation_email(&email_client, new_subscriber, &application_base_url.0)
        .await
        .is_err()
    {
        return HttpResponse::InternalServerError().finish();
    };

    HttpResponse::Created().finish()
}

#[tracing::instrument(
    name = "Send a confirmation email to a new subscriber",
    skip(email_client, subscriber_data, application_base_url)
)]
async fn send_confirmation_email(
    email_client: &EmailClient,
    subscriber_data: NewSubscriber,
    application_base_url: &str,
) -> Result<(), reqwest::Error> {
    let confirmation_link = format!(
        "{}/subscriptions/confirm?subscription_token={}",
        application_base_url, "my-temporal-token"
    );

    let plain_text_body = &format!(
        "Welcome to our newsletter!\nVisit {} to confirm your subscription.",
        confirmation_link
    );
    let html_body = &format!(
        "<p>Welcome to our newsletter!</p><br />\
        <p>Click <a href=\"{}\">here</a> to confirm your subscription.</p>",
        confirmation_link
    );

    email_client
        .send_email(subscriber_data.email, "Welcome", html_body, plain_text_body)
        .await
}

#[tracing::instrument(
    name = "Save new subscriber details in the database",
    skip(db_connection_pool, subscriber_data)
)]
async fn insert_subscriber_into_database(
    db_connection_pool: &PgPool,
    subscriber_data: &NewSubscriber,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at, status)
        VALUES ($1, $2, $3, $4, 'pending_confirmation')
        "#,
        Uuid::new_v4(),
        subscriber_data.email.as_ref(),
        subscriber_data.name.as_ref(),
        Utc::now()
    )
    .execute(db_connection_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

    Ok(())
}
