use std::net::TcpListener;

use actix_web;
use sqlx::PgPool;
use zero2prod::{
    configuration::get_configuration,
    startup::run,
    telemetry::{get_subscriber, init_subscriber},
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("zero2prod".into(), "info".into());
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("failed to read configuration");

    // application
    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listner = TcpListener::bind(address)?;

    // database connection
    let connection_string = configuration.database.connection_string();
    let connection = PgPool::connect(&connection_string)
        .await
        .expect("database connection failed");
    run(listner, connection)?.await
}
