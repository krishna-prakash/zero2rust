use std::net::TcpListener;

use actix_web::{dev::Server, App, HttpServer};

use crate::routes::{health_check, subscribe};

pub fn run(listner: TcpListener) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(|| App::new().service(subscribe).service(health_check))
        .listen(listner)?
        .run();

    Ok(server)
}
