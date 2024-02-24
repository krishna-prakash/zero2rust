use actix_web::{get, web, HttpResponse};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Deserialize, Debug)]
pub struct Parameters {
    subscription_token: String,
}

#[tracing::instrument(name = "confirm a pending subscriber", skip(parameters))]
#[get("/subscriptions/confirm")]
pub async fn confirm(
    parameters: web::Query<Parameters>,
    connection: web::Data<PgPool>,
) -> HttpResponse {
    let id = match get_subscriber_id_from_token(&connection, &parameters.subscription_token).await {
        Ok(id) => id,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    match id {
        None => HttpResponse::Unauthorized().finish(),
        Some(subscriber_id) => {
            if confirm_subscriber(&connection, subscriber_id)
                .await
                .is_err()
            {
                return HttpResponse::InternalServerError().finish();
            }
            HttpResponse::Ok().finish()
        }
    }
}

#[tracing::instrument(name = "Mark subscriber as confirmed", skip(connection, subscriber_id))]
pub async fn confirm_subscriber(
    connection: &PgPool,
    subscriber_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"UPDATE subscriptions SET status = 'confirmed' WHERE id = $1"#,
        subscriber_id
    )
    .execute(connection)
    .await
    .map_err(|e| {
        tracing::error!("falied to execute update query: {}", e);
        e
    })?;
    Ok(())
}

#[tracing::instrument(
    name = "get subscriber id from token",
    skip(connection, subscription_token)
)]
pub async fn get_subscriber_id_from_token(
    connection: &PgPool,
    subscription_token: &str,
) -> Result<Option<Uuid>, sqlx::Error> {
    let result = sqlx::query!(
        r#"SELECT subscriber_id FROM subscription_tokens WHERE subscription_token = $1"#,
        subscription_token
    )
    .fetch_optional(connection)
    .await
    .map_err(|e| {
        tracing::error!("falied to execute query {}", e);
        e
    })?;
    Ok(result.map(|r| r.subscriber_id))
}
