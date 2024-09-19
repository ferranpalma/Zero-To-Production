use actix_web::{post, web, HttpResponse};
use anyhow::Context;
use askama_actix::Template;
use chrono::Utc;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use serde::Deserialize;
use sqlx::{Executor, PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::{
    email_client::EmailClient,
    errors::{StoreTokenError, SubscribeError},
    models::NewSubscriber,
    startup::ApplicationBaseUrl,
    templates::ConfirmationEmailTemplate,
};

#[derive(Deserialize)]
pub struct SubscriberData {
    pub email: String,
    pub name: String,
}

fn create_subscription_token() -> String {
    let mut rng = thread_rng();
    std::iter::repeat_with(|| rng.sample(Alphanumeric))
        .map(char::from)
        .take(25)
        .collect()
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
) -> Result<HttpResponse, SubscribeError> {
    let new_subscriber = subscriber_data
        .0
        .try_into()
        .map_err(SubscribeError::ValidationError)?;

    let mut db_transaction = db_connection_pool
        .begin()
        .await
        .context("Failed to acquire a Postgres connection from the pool")?;
    let subscriber_id =
        insert_subscriber_into_database(&db_connection_pool, &mut db_transaction, &new_subscriber)
            .await
            .context("Failed to insert new subscriber in the database")?;

    let subscription_token = create_subscription_token();
    store_subscription_token_into_database(&mut db_transaction, subscriber_id, &subscription_token)
        .await
        .context("Failed to store the confirmation token for the new subscriber")?;

    db_transaction
        .commit()
        .await
        .context("Failed to commit the SQL transaction to store the new subscriber")?;

    send_confirmation_email(
        &email_client,
        new_subscriber,
        &application_base_url.0,
        &subscription_token,
    )
    .await
    .context("Failed to send confirmation email")?;

    Ok(HttpResponse::Created().finish())
}

#[tracing::instrument(
    name = "Store subscription token in the database",
    skip(subscription_token, db_transaction)
)]
async fn store_subscription_token_into_database(
    db_transaction: &mut Transaction<'_, Postgres>,
    subscriber_id: Uuid,
    subscription_token: &str,
) -> Result<(), StoreTokenError> {
    let db_query = sqlx::query!(
        r#"INSERT INTO subscription_tokens (subscription_token, subscriber_id)
        VALUES ($1, $2)
        "#,
        subscription_token,
        subscriber_id
    );
    db_transaction.execute(db_query).await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        StoreTokenError(e)
    })?;

    Ok(())
}

#[tracing::instrument(
    name = "Send a confirmation email to a new subscriber",
    skip(
        email_client,
        subscriber_data,
        application_base_url,
        subscription_token
    )
)]
async fn send_confirmation_email(
    email_client: &EmailClient,
    subscriber_data: NewSubscriber,
    application_base_url: &str,
    subscription_token: &str,
) -> Result<(), reqwest::Error> {
    let confirmation_link = format!(
        "{}/subscriptions/confirm?subscription_token={}",
        application_base_url, subscription_token
    );

    let plain_text_body = &format!(
        "Welcome to our newsletter!\nVisit {} to confirm your subscription.",
        confirmation_link
    );
    let html_body = ConfirmationEmailTemplate {
        confirmation_link: &confirmation_link,
    };
    let html_body = html_body
        .render()
        .expect("Failed to render html for confirmation email");

    email_client
        .send_email(
            &subscriber_data.email,
            "Welcome",
            &html_body,
            plain_text_body,
        )
        .await
}

#[tracing::instrument(
    name = "Save new subscriber details in the database",
    skip(db_transaction, subscriber_data)
)]
async fn insert_subscriber_into_database(
    db_connection_pool: &PgPool,
    db_transaction: &mut Transaction<'_, Postgres>,
    subscriber_data: &NewSubscriber,
) -> Result<Uuid, sqlx::Error> {
    let subscriber_id = Uuid::new_v4();

    let subscriber = sqlx::query!(
        r#"SELECT id FROM subscriptions WHERE email = $1 AND status = 'pending_confirmation'"#,
        subscriber_data.email.as_ref()
    )
    .fetch_optional(db_connection_pool)
    .await?;
    if let Some(subscriber) = subscriber {
        return Ok(subscriber.id);
    }

    let db_query = sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at, status)
        VALUES ($1, $2, $3, $4, 'pending_confirmation')
        "#,
        subscriber_id,
        subscriber_data.email.as_ref(),
        subscriber_data.name.as_ref(),
        Utc::now()
    );

    db_transaction.execute(db_query).await.map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

    Ok(subscriber_id)
}
