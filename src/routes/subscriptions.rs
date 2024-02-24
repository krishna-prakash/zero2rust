use actix_web::{post, web, HttpResponse};
use chrono::Utc;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use sqlx::{PgPool, Postgres, Transaction};
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

    let mut transaction = match connection.begin().await {
        Ok(transaction) => transaction,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    let subscriber_id = match insert_subscriber(&new_subscriber, &mut transaction).await {
        Ok(subscriber_id) => subscriber_id,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    let subscription_token = generate_subscription_token();
    if store_token(&mut transaction, subscriber_id, &subscription_token)
        .await
        .is_err()
    {
        return HttpResponse::InternalServerError().finish();
    };

    if transaction.commit().await.is_err() {
        return HttpResponse::InternalServerError().finish();
    };

    if send_confirmation_email(new_subscriber, &email_client, &subscription_token)
        .await
        .is_err()
    {
        return HttpResponse::InternalServerError().finish();
    }

    HttpResponse::Ok().finish()
}

fn generate_subscription_token() -> String {
    let mut rng = thread_rng();
    std::iter::repeat_with(|| rng.sample(Alphanumeric))
        .map(char::from)
        .take(25)
        .collect()
}

#[tracing::instrument(
    name = "sending confirmation email",
    skip(email_client, new_subscriber)
)]
pub async fn send_confirmation_email(
    new_subscriber: NewSubscriber,
    email_client: &EmailClient,
    subscription_token: &str,
) -> Result<(), reqwest::Error> {
    let confirmation_link = format!(
        "http://localhost:8000/subscriptions/confirm?subscription_token={}",
        subscription_token
    );

    email_client
        .send_email(
            new_subscriber.email,
            "welcome!",
            &format!("click <a href={}>here</a> to confirm", confirmation_link),
        )
        .await
}

#[tracing::instrument(name = "inserting subscriber", skip(form, connection))]
async fn insert_subscriber(
    form: &NewSubscriber,
    connection: &mut Transaction<'_, Postgres>,
) -> Result<Uuid, sqlx::Error> {
    let subscriber_id = Uuid::new_v4();

    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at, status)
        VALUES ($1, $2, $3, $4, 'pending_confirmation')
    "#,
        subscriber_id,
        form.email.as_ref(),
        form.name.as_ref(),
        Utc::now()
    )
    .execute(&mut **connection)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;

    Ok(subscriber_id)
}

#[tracing::instrument(
    name = "storing subscription token",
    skip(connection, subscriber_id, subscription_token)
)]
async fn store_token(
    connection: &mut Transaction<'_, Postgres>,
    subscriber_id: Uuid,
    subscription_token: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"INSERT INTO SUBSCRIPTION_TOKENS (subscription_token, subscriber_id) VALUES ($1, $2)"#,
        subscription_token,
        subscriber_id
    )
    .execute(&mut **connection)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {}", e);
        e
    })?;
    Ok(())
}
