use utoipa::OpenApi;
use axum::http::StatusCode;
use axum::Json;
use axum::response::{Html, IntoResponse};
use crate::routes::ApiDocs;

pub async fn serve_api_docs() -> impl IntoResponse {
  (StatusCode::OK, Json(ApiDocs::openapi()))
}

pub async fn serve_swagger() -> Html<String> {
  Html(include_str!("../../static/swagger-ui/index.html").to_string()) 
}
