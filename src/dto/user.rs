use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Deserialize, Serialize, ToSchema)]
pub struct CreateUserDto {
  pub name: String,
  pub email: String,
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct UpdateUserDto {
  pub name: Option<String>,
  pub email: Option<String>,
}

#[derive(Deserialize, Serialize, ToSchema)]
pub struct UserResponseDto {
  pub id: String,
  pub name: String,
  pub email: String,
}