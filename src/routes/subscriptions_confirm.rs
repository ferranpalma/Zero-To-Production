use actix_web::{get, http::StatusCode, web, HttpResponse, ResponseError};
use anyhow::Context;
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::SubscriptionToken;

#[derive(serde::Deserialize)]
pub struct QueryParameters {
    pub subscription_token: String,
}

#[derive(thiserror::Error)]
pub enum ConfirmError {
    #[error("{0}")]
    TokenError(String),
    #[error("There is no subscriber associated with the provided token")]
    UnknownToken,
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl ResponseError for ConfirmError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            ConfirmError::TokenError(_) => StatusCode::BAD_REQUEST,
            ConfirmError::UnknownToken => StatusCode::UNAUTHORIZED,
            ConfirmError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

fn format_error_chain(
    e: &impl std::error::Error,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    writeln!(f, "{}\n", e)?;
    let mut current_error = e.source();
    while let Some(cause) = current_error {
        writeln!(f, "Caused by:\n\t{}", cause)?;
        current_error = cause.source();
    }
    Ok(())
}

impl std::fmt::Debug for ConfirmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        format_error_chain(self, f)
    }
}

#[tracing::instrument(name = "Confirm subscriber", skip(db_connection_pool, queryparams))]
#[get("/subscriptions/confirm")]
pub async fn confirm_subscriber(
    db_connection_pool: web::Data<PgPool>,
    queryparams: web::Query<QueryParameters>,
) -> Result<HttpResponse, ConfirmError> {
    let subscription_token = SubscriptionToken::parse(queryparams.subscription_token.clone())
        .map_err(ConfirmError::TokenError)
        .context("The token was invalid")?;

    let subscriber_id = get_subscriber_id_from_token(&db_connection_pool, &subscription_token)
        .await
        .context("No email was associated to this token")?
        .ok_or(ConfirmError::UnknownToken)?;

    mark_subscriber_status_as_confirmed(&db_connection_pool, subscriber_id)
        .await
        .context("Failed to mark subscriber as confirmed")?;

    Ok(HttpResponse::Ok().finish())
}

#[tracing::instrument(
    name = "Mark subscriber as confirmed",
    skip(db_connection_pool, subscriber_id)
)]
async fn mark_subscriber_status_as_confirmed(
    db_connection_pool: &PgPool,
    subscriber_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"UPDATE subscriptions SET status = 'confirmed' WHERE id = $1"#,
        subscriber_id
    )
    .execute(db_connection_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

    Ok(())
}

#[tracing::instrument(
    name = "Get subscriber id from associated token",
    skip(db_connection_pool, subscription_token)
)]
async fn get_subscriber_id_from_token(
    db_connection_pool: &PgPool,
    subscription_token: &SubscriptionToken,
) -> Result<Option<Uuid>, sqlx::Error> {
    let result = sqlx::query!(
        r#"SELECT subscriber_id FROM subscription_tokens WHERE subscription_token = $1"#,
        subscription_token.as_ref()
    )
    .fetch_optional(db_connection_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

    Ok(result.map(|r| r.subscriber_id))
}
