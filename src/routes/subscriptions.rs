use actix_web::{post, web, HttpResponse};
use chrono::Utc;
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{email_client::EmailClient, models::NewSubscriber};

#[derive(Deserialize)]
pub struct SubscriberData {
    pub email: String,
    pub name: String,
}

#[tracing::instrument(
    name = "Add a new subscriber",
    skip(subscriber_data, db_connection_pool, email_client),
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
) -> HttpResponse {
    let new_subscriber = match subscriber_data.0.try_into() {
        Ok(x) => x,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };

    dbg!("New subscriber created");

    if insert_subscriber_into_database(&db_connection_pool, &new_subscriber)
        .await
        .is_err()
    {
        return HttpResponse::InternalServerError().finish();
    };

    dbg!("New subscriber in the database");

    if email_client
        .send_email(new_subscriber.email, "Welcome", "Welcome", "Welcome")
        .await
        .is_err()
    {
        return HttpResponse::InternalServerError().finish();
    }

    dbg!("New subscriber mail sent");

    HttpResponse::Created().finish()
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
        VALUES ($1, $2, $3, $4, 'confirmed')
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
