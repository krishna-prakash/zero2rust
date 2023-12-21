use std::net::TcpListener;

use actix_web::{dev::Server, get, post, web, App, HttpResponse, HttpServer, Responder};

pub fn run(listner: TcpListener) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(|| App::new().service(subscribe).service(health_check))
        .listen(listner)?
        .run();

    Ok(server)
}
