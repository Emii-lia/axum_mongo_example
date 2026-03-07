#[utoipa::path(
  get,
  path = "/health",
  responses(
    (status = 200, description = "Health check"),
    (status = 500, description = "Internal Server Error")
  ),
  tag = "health"
)]
pub async fn health() -> &'static str {
  "OK"
}