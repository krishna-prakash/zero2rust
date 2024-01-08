use crate::helpers::spawn_app;

#[actix_web::test]
async fn confirmation_without_token_rejected() {
    let app = spawn_app().await;
    println!("{}", &format!("{}/subscriptions/confirm", app.address));
    let response = reqwest::get(&format!("http://{}/subscriptions/confirm", app.address))
        .await
        .unwrap();

    assert_eq!(response.status().as_u16(), 400);
}
