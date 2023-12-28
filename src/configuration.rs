use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Settings {
    pub database: Databasettings,
    pub application: ApplicationSettings,
}

#[derive(Deserialize, Debug)]
pub struct ApplicationSettings {
    pub port: u16,
    pub host: String,
}

#[derive(Deserialize, Debug)]
pub struct Databasettings {
    pub username: String,
    pub password: Secret<String>,
    pub port: u16,
    pub host: String,
    pub database_name: String,
}

pub enum Environment {
    Local,
    Production,
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Local => "local",
            Environment::Production => "production",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "production" => Ok(Self::Production),
            other => Err(format!("{} not a supported env", other)),
        }
    }
}

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let base_path = std::env::current_dir().expect("unable to get current directory");
    let config_dir = base_path.join("configuration");
    let environment: Environment = std::env::var("APP_ENVIRONMENT")
        .unwrap_or_else(|_| "local".into())
        .try_into()
        .expect("failed to parse env");

    let settings = config::Config::builder()
        .add_source(config::File::from(config_dir.join("base")))
        .add_source(config::File::from(config_dir.join(environment.as_str())))
        .build()
        .unwrap();

    settings.try_deserialize()
}

impl Databasettings {
    pub fn connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port,
            self.database_name
        )
    }

    pub fn connection_string_without_db(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/postgres",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port
        )
    }
}
