use crate::helpers::spawn_app;

#[actix_web::test]
async fn subscriber_returns_200_for_valid_post_data() {
    let app = spawn_app().await;

    let body = "name=krishna&email=krish2cric%40gmail.com";
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
