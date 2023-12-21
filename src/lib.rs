// use std::net::TcpListener;
//
// use actix_web::{dev::Server, get, post, web, App, HttpResponse, HttpServer, Responder};
//
// pub fn run(listner: TcpListener) -> Result<Server, std::io::Error> {
//     let server = HttpServer::new(|| App::new().service(subscribe).service(health_check))
//         .listen(listner)?
//         .run();
//
//     Ok(server)
// }
//
// #[get("/health_check")]
// async fn health_check() -> impl Responder {
//     HttpResponse::Ok()
// }
//
// #[derive(serde::Deserialize)]
// struct Subscription {
//     email: String,
//     name: String,
// }
//
// #[post("/subscription")]
// async fn subscribe(_req_body: web::Form<Subscription>) -> impl Responder {
//     HttpResponse::Ok()
// }
//

pub mod configuration;
pub mod routes;
pub mod startup;
