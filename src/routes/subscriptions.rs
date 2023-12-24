use actix_web::{post, web, HttpResponse};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(serde::Deserialize)]
struct Subscription {
    email: String,
    name: String,
}

#[post("/subscription")]
async fn subscribe(form: web::Form<Subscription>, connection: web::Data<PgPool>) -> HttpResponse {
    let request_id = Uuid::new_v4();
    let req_span = tracing::info_span!("request_id -> saving new subscriber", %request_id);
    let _req_span_guard = req_span.enter();

    match sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
    "#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
    .execute(connection.get_ref())
    .await
    {
        Ok(_) => {
            tracing::info_span!("request_id -> saved subscriber", %request_id);
            HttpResponse::Ok().finish()
        }
        Err(e) => {
            tracing::error_span!(
                "srequest_id -> saving subscriber failed: {:?}",
                %request_id,
                error = %e
            );
            HttpResponse::InternalServerError().finish()
        }
    }
}
