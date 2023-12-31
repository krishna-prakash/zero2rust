use actix_web::{post, web, HttpResponse};
use chrono::Utc;
use sqlx::PgPool;
use unicode_segmentation::UnicodeSegmentation;
use uuid::Uuid;

#[derive(serde::Deserialize)]
struct Subscription {
    email: String,
    name: String,
}

fn is_valid_name(s: &str) -> bool {
    let is_empty_or_whitespace = s.trim().is_empty();

    let is_too_long = s.graphemes(true).count() > 256;

    let forbidden_characters = ['(', ')', '/', '"', '<', '>', '\\', '{', '}'];
    let contains_forbidden_characters = s.chars().any(|g| forbidden_characters.contains(&g));
    println!(
        "{} {} {}",
        is_empty_or_whitespace, is_too_long, contains_forbidden_characters,
    );
    !(is_empty_or_whitespace || is_too_long || contains_forbidden_characters)
}

#[tracing::instrument(name = "Adding new subscriber", skip(form, connection))]
#[post("/subscription")]
async fn subscribe(form: web::Form<Subscription>, connection: web::Data<PgPool>) -> HttpResponse {
    if !is_valid_name(&form.name) {
        return HttpResponse::BadRequest().finish();
    }

    match insert_subscriber(&form, &connection).await {
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

async fn insert_subscriber(form: &Subscription, connection: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
    "#,
        Uuid::new_v4(),
        form.email,
        form.name,
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
