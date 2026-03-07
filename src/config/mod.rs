use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
  pub server: ServerConfig,
  pub database: DatabaseConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
  pub port: u16,
  pub host: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
  pub url: String,
  pub name: String,
}

impl Config {
  pub fn from_env() -> Result<Self, config::ConfigError> {
    dotenvy::dotenv().ok();
    
    let server = ServerConfig {
      host: std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
      port: std::env::var("PORT").unwrap_or_else(|_| "3001".to_string()).parse().unwrap_or(3001),
    };
    
    let database = DatabaseConfig {
      url: std::env::var("DATABASE_URL").unwrap_or_else(|_| "mongodb://localhost:27017".to_string()),
      name: std::env::var("DATABASE_NAME").unwrap_or_else(|_| "axum_user".to_string()),
    };
    
    Ok(Config { server, database })
  }
}

pub mod config {
  use std::error::Error;
  use std::fmt::Display;

  #[derive(Debug)]
  pub enum ConfigError {
    MissingEnvVar(String),
  }

  impl Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      match self {
        ConfigError::MissingEnvVar(var) => write!(f, "Missing environment variable: {}", var),
      }
    }
  }

  impl Error for ConfigError {}
}