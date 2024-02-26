use reqwest::Url;
use wiremock::{
    matchers::{method, path},
    Mock, ResponseTemplate,
};

use crate::helpers::spawn_app;

#[actix_web::test]
async fn subscriber_returns_200_for_valid_post_data() {
    let app = spawn_app().await;

    let body = "name=krishna&email=krish2cric%40gmail.com";
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;
    let response = app.post_subscription(body.into()).await;
    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT email, name FROM subscriptions")
        .fetch_one(&app.db_pool)
        .await
        .expect("query failed");

    assert_eq!(saved.email, "krish2cric@gmail.com");
    assert_eq!(saved.name, "krishna");
}

#[actix_web::test]
async fn subscriber_valid_post_data_saved() {
    let app = spawn_app().await;

    let body = "name=krishna&email=krish2cric%40gmail.com";
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;
    let _ = app.post_subscription(body.into()).await;

    let saved = sqlx::query!("SELECT email, name, status FROM subscriptions")
        .fetch_one(&app.db_pool)
        .await
        .expect("query failed");

    assert_eq!(saved.email, "krish2cric@gmail.com");
    assert_eq!(saved.name, "krishna");
    assert_eq!(saved.status, "pending_confirmation");
}

#[actix_web::test]
async fn subscriber_returns_400_for_invalid_post_data() {
    let app = spawn_app().await;

    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("name=krishna&email=", "missing the email"),
        ("email=krish2cric@gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_data, error) in test_cases {
        let response = app.post_subscription(invalid_data.into()).await;
        assert_eq!(400, response.status().as_u16(), "API error {}", error);
    }
}

#[actix_web::test]
async fn subscribe_sends_email_for_valid_data() {
    let app = spawn_app().await;

    let body = "name=krishna&email=krish2cric%40gmail.com";
    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    app.post_subscription(body.into()).await;

    let email_req = &app.email_server.received_requests().await.unwrap()[0];

    let body: serde_json::Value = serde_json::from_slice(&email_req.body).unwrap();
    let get_link = |s: &str| {
        let links: Vec<_> = linkify::LinkFinder::new()
            .links(s)
            .filter(|l| *l.kind() == linkify::LinkKind::Url)
            .collect();

        assert_eq!(links.len(), 1);
        links[0].as_str().to_owned()
    };

    let raw_confirmation_link = get_link(&body["htmlContent"].as_str().unwrap());
    let mut confirmation_link = Url::parse(&raw_confirmation_link).unwrap();

    assert_eq!(confirmation_link.host_str().unwrap(), "localhost");
    confirmation_link.set_port(Some(app.port)).unwrap();

    let response = reqwest::get(confirmation_link).await.unwrap();

    assert_eq!(response.status().as_u16(), 200);
}

#[actix_web::test]
async fn subscribe_fails_if_there_is_a_fatal_database_error() {
    let app = spawn_app().await;
    let body = "name=krishna&email=krish2cric%40gmail.com";

    sqlx::query!("ALTER TABLE subscription_tokens DROP COLUMN subscription_token")
        .execute(&app.db_pool)
        .await
        .unwrap();

    let response = app.post_subscription(body.into()).await;

    assert_eq!(response.status().as_u16(), 500);
}
