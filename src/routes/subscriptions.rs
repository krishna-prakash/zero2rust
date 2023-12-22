use actix_web::{post, web, HttpResponse, Responder};

#[derive(serde::Deserialize)]
struct Subscription {
    email: String,
    name: String,
}

#[post("/subscription")]
async fn subscribe(_req_body: web::Form<Subscription>) -> impl Responder {
    HttpResponse::Ok()
}
