use std::net::TcpListener;

use actix_web::{self};

#[actix_web::test]
async fn health_check_works() {
    let address = spawn_app();
    let client = reqwest::Client::new();
    let response = client
        .get(&format!("http://{}/health_check", &address))
        .send()
        .await
        .expect("failed to execute request");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[actix_web::test]
async fn subscriber_returns_200_for_valid_post_data() {
    let app = spawn_app();

    let client = reqwest::Client::new();
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = client
        .post(&format!("http://{}/subscription", &app))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(200, response.status().as_u16());
}

#[actix_web::test]
async fn subscriber_returns_400_for_invalid_post_data() {
    let app = spawn_app();

    let client = reqwest::Client::new();

    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_data, error) in test_cases {
        let response = client
            .post(&format!("http://{}/subscription", &app))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_data)
            .send()
            .await
            .expect("Failed to execute request");

        assert_eq!(400, response.status().as_u16(), "API error {}", error);
    }
}

fn spawn_app() -> String {
    let listner = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listner.local_addr().unwrap().port();
    let server = zero2prod::run(listner).expect("Failed to bind address");
    let _ = tokio::spawn(server);

    format!("127.0.0.1:{}", port)
}
