use mongodb::Database;
use crate::config::Config;

#[derive(Clone)]
pub struct AppState {
  pub db: Database,
  pub config: Config
}

impl AppState {
  pub fn new(db: Database, config: Config) -> Self {
    Self { db, config }
  }
}