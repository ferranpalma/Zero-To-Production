use actix_web::{post, web, HttpResponse};
use anyhow::Context;
use sqlx::PgPool;

use crate::{email_client::EmailClient, errors::PublishError, models::SubscriberEmail};

#[derive(serde::Deserialize)]
struct EmailBodyData {
    title: String,
    content: EmailContentData,
}

#[derive(serde::Deserialize)]
struct EmailContentData {
    html: String,
    plain_text: String,
}

struct ConfirmedSubscriber {
    email: SubscriberEmail,
}

#[tracing::instrument(name = "Get confirmed subscribers", skip(db_connection_pool))]
async fn get_confirmed_subscribers(
    db_connection_pool: &PgPool,
) -> Result<Vec<Result<ConfirmedSubscriber, anyhow::Error>>, anyhow::Error> {
    let confirmed_subscribers =
        sqlx::query!(r#"SELECT email FROM subscriptions WHERE status = 'confirmed'"#,)
            .fetch_all(db_connection_pool)
            .await?
            .into_iter()
            .map(|s| match SubscriberEmail::parse(s.email) {
                Ok(email) => Ok(ConfirmedSubscriber { email }),
                Err(e) => Err(anyhow::anyhow!(e)),
            })
            .collect();

    Ok(confirmed_subscribers)
}

#[post("/newsletters")]
pub async fn publish_newsletter(
    db_connection_pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
    email_body: web::Json<EmailBodyData>,
) -> Result<HttpResponse, PublishError> {
    let confirmed_subscribers = get_confirmed_subscribers(&db_connection_pool).await?;
    for confirmed_subscriber in confirmed_subscribers {
        match confirmed_subscriber {
            Ok(s) => {
                email_client
                    .send_email(
                        &s.email,
                        &email_body.title,
                        &email_body.content.html,
                        &email_body.content.plain_text,
                    )
                    .await
                    .with_context(|| format!("Failed to send newsletter issue to {}", s.email))?;
            }
            Err(e) => {
                tracing::warn!(e.cause_chain = ?e, "Skipping a confirmed subscriber. Their stored email is invalid");
            }
        }
    }
    Ok(HttpResponse::Ok().finish())
}
