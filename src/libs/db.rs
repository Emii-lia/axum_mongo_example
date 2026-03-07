use mongodb::{Client, Database};
use mongodb::options::ClientOptions;
use crate::config::Config;

pub async fn init_db(config: Config) -> Result<Database, Box<dyn std::error::Error>> {
  let client_options = ClientOptions::parse(config.database.url.as_str()).await?;
  let client = Client::with_options(client_options)?;

  Ok(client.database(config.database.name.as_str()))
}