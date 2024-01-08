use actix_web::{post, web, HttpResponse};
use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    domain::{NewSubscriber, Subscription},
    email_client::EmailClient,
};

#[tracing::instrument(name = "Adding new subscriber", skip(form, connection))]
#[post("/subscription")]
async fn subscribe(
    form: web::Form<Subscription>,
    connection: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
) -> HttpResponse {
    let new_subscriber = match form.0.try_into() {
        Ok(form) => form,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };

    if insert_subscriber(&new_subscriber, &connection)
        .await
        .is_err()
    {
        return HttpResponse::InternalServerError().finish();
    };

    if send_confirmation_email(new_subscriber, &email_client)
        .await
        .is_err()
    {
        return HttpResponse::InternalServerError().finish();
    }

    HttpResponse::Ok().finish()
}

#[tracing::instrument(
    name = "sending confirmation email",
    skip(email_client, new_subscriber)
)]
pub async fn send_confirmation_email(
    new_subscriber: NewSubscriber,
    email_client: &EmailClient,
) -> Result<(), reqwest::Error> {
    let confirmation_link = "http://localhost:8000/subscriptions/confirm";

    email_client
        .send_email(
            new_subscriber.email,
            "welcome!",
            &format!("click <a href={}>here</a> to confirm", confirmation_link),
        )
        .await
}

#[tracing::instrument(name = "inserting subscriber", skip(form, connection))]
async fn insert_subscriber(form: &NewSubscriber, connection: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at, status)
        VALUES ($1, $2, $3, $4, 'pending_confirmation')
    "#,
        Uuid::new_v4(),
        form.email.as_ref(),
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
