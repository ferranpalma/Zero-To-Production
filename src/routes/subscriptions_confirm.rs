use actix_web::{get, web, HttpResponse};

#[derive(serde::Deserialize)]
pub struct QueryParameters {
    subscription_token: String,
}

#[tracing::instrument(name = "Confirm subscriber", skip(_queryparams))]
#[get("/subscriptions/confirm")]
pub async fn confirm_subscriber(_queryparams: web::Query<QueryParameters>) -> HttpResponse {
    HttpResponse::Ok().finish()
}
