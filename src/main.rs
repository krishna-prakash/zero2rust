use std::net::TcpListener;

use actix_web;
use sqlx::postgres::PgPoolOptions;
use zero2prod::{
    configuration::get_configuration,
    startup::run,
    telemetry::{get_subscriber, init_subscriber},
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("failed to read configuration");

    // application
    let address = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );
    let listner = TcpListener::bind(address)?;

    // database connection
    let connection_string = configuration.database.connection_string();
    let connection = PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy(&connection_string)
        .expect("database connection failed");
    run(listner, connection)?.await
}
