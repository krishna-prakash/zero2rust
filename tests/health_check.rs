use std::net::TcpListener;

use actix_web;

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

fn spawn_app() -> String {
    let listner = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listner.local_addr().unwrap().port();
    let server = zero2prod::run(listner).expect("Failed to bind address");
    let _ = tokio::spawn(server);

    format!("127.0.0.1:{}", port)
}
