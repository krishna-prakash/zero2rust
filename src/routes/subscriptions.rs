use actix_web::{post, web, HttpResponse};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::{NewSubscriber, SubscriberName};

#[derive(serde::Deserialize)]
struct Subscription {
    email: String,
    name: String,
}

#[tracing::instrument(name = "Adding new subscriber", skip(form, connection))]
#[post("/subscription")]
async fn subscribe(form: web::Form<Subscription>, connection: web::Data<PgPool>) -> HttpResponse {
    let new_subscriber = NewSubscriber {
        name: SubscriberName::parse(form.0.name),
        email: form.0.email,
    };

    match insert_subscriber(&new_subscriber, &connection).await {
        Ok(_) => {
            tracing::info_span!("saved subscriber");
            HttpResponse::Ok().finish()
        }
        Err(e) => {
            tracing::error!("saving subscriber failed: {:?}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[tracing::instrument(name = "inserting subscriber", skip(form, connection))]

async fn insert_subscriber(form: &NewSubscriber, connection: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
    "#,
        Uuid::new_v4(),
        form.email,
        form.name.as_ref(),
        Utc::now()
    )
    .execute(connection)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

    Ok(())
}
