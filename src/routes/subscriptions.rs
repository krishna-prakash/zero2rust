use actix_web::{dev::Server, get, post, web, App, HttpResponse, HttpServer, Responder};

#[derive(serde::Deserialize)]
struct Subscription {
    email: String,
    name: String,
}

#[post("/subscription")]
async fn subscribe(_req_body: web::Form<Subscription>) -> impl Responder {
    HttpResponse::Ok()
}
