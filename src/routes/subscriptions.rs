use actix_web::{post, web, HttpResponse};
use chrono::Utc;
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::NewSubscriber;

#[derive(Deserialize)]
pub struct SubscriberData {
    pub email: String,
    pub name: String,
}

#[tracing::instrument(
    name = "Add a new subscriber",
    skip(subscriber_data, db_connection_pool),
    fields(
        subscriber_name = %subscriber_data.name,
        subscriber_email = %subscriber_data.email,
        )
)]
#[post("/subscriptions")]
pub async fn subscribe(
    subscriber_data: web::Form<SubscriberData>,
    db_connection_pool: web::Data<PgPool>,
) -> HttpResponse {
    let new_subscriber = match subscriber_data.0.try_into() {
        Ok(x) => x,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };

    match insert_subscriber_into_database(&db_connection_pool, &new_subscriber).await {
        Ok(_) => HttpResponse::Created().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
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
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
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
