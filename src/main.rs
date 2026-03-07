use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use crate::config::Config;
use crate::libs::db::init_db;
use crate::routes::create_router;
use crate::state::AppState;

mod config;
mod libs;
mod model;
mod dto;
mod handler;
mod state;
mod routes;

#[tokio::main]
async fn main() {
  let config = Config::from_env().expect("Failed to load config");
  let db = init_db(config.clone()).await.expect("Failed to connect to database");

  let state = Arc::new(AppState::new(db, config));

  let app = create_router(state.clone())
    .layer(TraceLayer::new_for_http()
      .make_span_with(DefaultMakeSpan::default().include_headers(true))
    )
    .layer(CorsLayer::permissive());

  let listener = {
    let state = state.clone();
    TcpListener::bind(format!("{}:{}", state.config.server.host, state.config.server.port).as_str())
    .await.unwrap()
  };

  axum::serve(listener, app).await.unwrap();
}
