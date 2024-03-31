use actix_web::{
    web::{Data, Form},
    HttpResponse,
};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct SubscriptionData {
    email: String,
    name: String,
}

pub async fn subscribe(form: Form<SubscriptionData>, connection: Data<PgPool>) -> HttpResponse {
    let request_id = Uuid::new_v4();
    tracing::info!(
        "request {} | Adding '{}' '{}' as a new subscriber.",
        request_id,
        form.email,
        form.name
    );
    tracing::info!(
        "request {} | Saving new subscriber details in the database",
        request_id
    );
    match sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now(),
    )
    .execute(connection.as_ref())
    .await
    {
        Ok(_) => {
            tracing::info!(
                "request {} | Saving new subscriber details in the database",
                request_id
            );
            HttpResponse::Ok().finish()
        }
        Err(e) => {
            tracing::error!(
                "request {} | Failed to execute the query: {:?}",
                request_id,
                e
            );
            HttpResponse::InternalServerError().finish()
        }
    }
}
