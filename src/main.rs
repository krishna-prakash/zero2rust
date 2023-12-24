use std::net::TcpListener;

use actix_web;
use sqlx::PgPool;
use tracing::subscriber::set_global_default;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_log::LogTracer;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};
use zero2prod::{configuration::get_configuration, startup::run};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    LogTracer::init().expect("Log init failed");
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let formatting_layer = BunyanFormattingLayer::new("zero2prod".into(), std::io::stdout);

    let subscriber = Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer);

    set_global_default(subscriber).expect("Failed to set subscriber");

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
