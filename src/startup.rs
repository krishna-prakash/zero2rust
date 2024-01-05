use std::net::TcpListener;

use actix_web::{dev::Server, web, App, HttpServer};
use sqlx::{postgres::PgPoolOptions, PgPool};
use tracing_actix_web::TracingLogger;

use crate::{
    configuration::{Databasettings, Settings},
    email_client::EmailClient,
    routes::{health_check, subscribe},
};

pub struct Application {
    port: u16,
    server: Server,
}

impl Application {
    pub async fn build(configuration: Settings) -> Result<Self, std::io::Error> {
        // application
        let address = format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        );
        let listner = TcpListener::bind(address)?;
        let port = listner.local_addr().unwrap().port();

        let connection = get_connection_pool(&configuration.database);
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

        let server = run(listner, connection, email_client)?;

        Ok(Self { port, server })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

pub fn get_connection_pool(configuration: &Databasettings) -> PgPool {
    PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(20))
        .connect_lazy_with(configuration.with_db())
}

pub fn run(
    listner: TcpListener,
    connection: PgPool,
    email_client: EmailClient,
) -> Result<Server, std::io::Error> {
    let connection = web::Data::new(connection);
    let email_client = web::Data::new(email_client);
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .service(subscribe)
            .service(health_check)
            .app_data(connection.clone())
            .app_data(email_client.clone())
    })
    .listen(listner)?
    .run();

    Ok(server)
}
