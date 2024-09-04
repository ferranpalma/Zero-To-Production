use actix_web::{post, web, HttpResponse};
use serde::Deserialize;

#[derive(Deserialize)]
struct SubscriberData {
    email: String,
    name: String,
}

#[post("/subscriptions")]
pub async fn subscribe(_form: web::Form<SubscriberData>) -> HttpResponse {
    HttpResponse::Ok().finish()
}
