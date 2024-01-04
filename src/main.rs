use std::net::TcpListener;

use actix_web;
use sqlx::postgres::PgPoolOptions;
use zero2prod::{
    configuration::get_configuration,
    email_client::EmailClient,
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
    let connection_string = configuration.database.with_db();

    let connection = PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(20))
        .connect_lazy_with(connection_string);

    let email_sender = configuration
        .email_client
        .sender()
        .expect("not a valid email");

    let timeout = configuration.email_client.timeout();
    let auth_token = configuration.email_client.auth_token;

    let email_client = EmailClient::new(
        configuration.email_client.base_url,
        email_sender,
        auth_token,
        timeout,
    );
    run(listner, connection, email_client)?.await
}
