use std::net::TcpListener;

use actix_web::{dev::Server, get, post, App, HttpResponse, HttpServer, Responder};

pub fn run(listner: TcpListener) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(|| {
        App::new()
            .service(hello)
            .service(echo)
            .service(health_check)
    })
    .listen(listner)?
    .run();

    Ok(server)
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello actix")
}

#[get("/health_check")]
async fn health_check() -> impl Responder {
    HttpResponse::Ok()
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}
