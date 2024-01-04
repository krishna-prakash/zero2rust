use std::net::TcpListener;

use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;
use zero2prod::{
    configuration::{get_configuration, Databasettings},
    email_client::EmailClient,
    telemetry::{get_subscriber, init_subscriber},
};

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();

    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    }
});

pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    let listner = TcpListener::bind("127.0.0.1:0").expect("Failed to bind random port");
    let port = listner.local_addr().unwrap().port();

    let mut configuration = get_configuration().expect("failed to read configuration");
    configuration.database.database_name = Uuid::new_v4().to_string();
    let connection_pool = configure_database(&configuration.database).await;

    let email_sender = configuration.email_client.sender().expect("invalid email");
    let auth_token = configuration.email_client.auth_token;
    let timeout = std::time::Duration::from_millis(200);
    let email_client = EmailClient::new(
        configuration.email_client.base_url,
        email_sender,
        auth_token,
        timeout,
    );

    let server = zero2prod::startup::run(listner, connection_pool.clone(), email_client)
        .expect("Failed to bind address");
    let _ = tokio::spawn(server);

    let address = format!("127.0.0.1:{}", port);

    TestApp {
        address,
        db_pool: connection_pool,
    }
}

pub async fn configure_database(config: &Databasettings) -> PgPool {
    let mut connection = PgConnection::connect_with(&config.without_db())
        .await
        .expect("connection failed");

    println!(
        "{}",
        format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str()
    );
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Database creation failed");

    let connection_pool = PgPool::connect_with(config.with_db())
        .await
        .expect("db connection failed");

    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("migrations failed");

    connection_pool
}
