use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::dto::user::UserResponseDto;
use crate::model::user::User;

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
  pub owner: Option<UserResponseDto>,
}

#[derive(Serialize, Deserialize)]
pub struct CarDetailsDto {
  #[serde(rename = "_id")]
  pub id: ObjectId,
  pub name: String,
  pub owner: Option<User>,
}