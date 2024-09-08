use actix_web::{get, web, HttpResponse};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct QueryParameters {
    subscription_token: String,
}

#[tracing::instrument(name = "Confirm subscriber", skip(db_connection_pool, queryparams))]
#[get("/subscriptions/confirm")]
pub async fn confirm_subscriber(
    db_connection_pool: web::Data<PgPool>,
    queryparams: web::Query<QueryParameters>,
) -> HttpResponse {
    let subscriber_id =
        match get_subscriber_id_from_token(&db_connection_pool, &queryparams.subscription_token)
            .await
        {
            Ok(id) => id,
            Err(_) => return HttpResponse::InternalServerError().finish(),
        };

    match subscriber_id {
        None => return HttpResponse::Unauthorized().finish(),
        Some(subscriber_id) => {
            if mark_subscriber_status_as_confirmed(&db_connection_pool, subscriber_id)
                .await
                .is_err()
            {
                return HttpResponse::InternalServerError().finish();
            }
            HttpResponse::Ok().finish()
        }
    }
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
    subscription_token: &str,
) -> Result<Option<Uuid>, sqlx::Error> {
    let result = sqlx::query!(
        r#"SELECT subscriber_id FROM subscription_tokens WHERE subscription_token = $1"#,
        subscription_token
    )
    .fetch_optional(db_connection_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

    Ok(result.map(|r| r.subscriber_id))
}
