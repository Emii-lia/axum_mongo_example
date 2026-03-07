use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Deserialize, Serialize, ToSchema)]
pub struct CreateCarDto {
  pub name: String,
  pub owner_id: String,
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct UpdateCarDto {
  pub name: Option<String>,
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct CarResponseDto {
  pub id: String,
  pub name: String,
  pub owner_id: String,
}