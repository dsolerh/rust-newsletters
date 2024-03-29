use actix_web::{web::Form, HttpResponse};

#[derive(serde::Deserialize)]
pub struct SubscriptionData {
    email: String,
    name: String,
}

pub async fn subscribe(_form: Form<SubscriptionData>) -> HttpResponse {
    HttpResponse::Ok().finish()
}
