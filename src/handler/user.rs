use futures_util::stream::TryStreamExt;
use std::sync::Arc;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use mongodb::bson::{doc, oid::ObjectId};
use mongodb::Collection;
use crate::dto::user::{CreateUserDto, UpdateUserDto, UserResponseDto};
use crate::model::user::User;
use crate::state::AppState;

#[utoipa::path(
  post,
  path = "/api/users",
  request_body = CreateUserDto,
  responses(
    (status = 201, description = "User Added successfully", body = UserResponseDto),
    (status = 404, description = "User not found"),
    (status = 500, description = "Internal Server Error")
  ),
  tag = "users"
)]
#[axum::debug_handler]
pub async fn create_user(
  State(state): State<Arc<AppState>>,
  Json(payload): Json<CreateUserDto>
) -> Result<(StatusCode, Json<UserResponseDto>), (StatusCode, String)> {
  let collection = state.db.collection("users");

  let new_user = User {
    id: None,
    name: payload.name,
    email: payload.email,
  };

  match collection.insert_one(new_user, None).await {
    Ok(result) => {
      let user_id = result.inserted_id.as_object_id().unwrap();
      let created_user = collection
        .find_one(doc! { "_id": user_id}, None)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch user".to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "User not found".to_string()))?;

      let user = UserResponseDto {
        id: created_user.id.unwrap().to_string(),
        name: created_user.name,
        email: created_user.email,
      };
      Ok((StatusCode::CREATED, Json(user)))
    }
    Err(_) => {
      Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to create user".to_string()))
    }
  }
}

#[utoipa::path(
  get,
  path = "/api/users",
  responses(
    (status = 200, description = "List of users", body = Vec<UserResponseDto>),
    (status = 500, description = "Internal Server Error")
  ),
  tag = "users"
)]
#[axum::debug_handler]
pub async fn list_users(
  State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<UserResponseDto>>, (StatusCode, String)> {
  let collection = state.db.collection("users");

  match collection.find(None, None).await {
    Ok(cursor) => {
      let docs: Vec<User> = cursor.try_collect()
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch users".to_string()))?;

      let users: Vec<UserResponseDto> = docs.into_iter().map(|user| UserResponseDto {
        id: user.id.unwrap().to_string(),
        name: user.name,
        email: user.email,
      }).collect();

      Ok(Json(users))
    }
    Err(_) => {
      Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch users".to_string()))
    }
  }
}

#[utoipa::path(
  get,
  path = "/api/users/{id}",
  params(
    ( "id" = String, Path, description = "User ID")
  ),
  responses(
    (status = 200, description = "User details", body = UserResponseDto),
    (status = 404, description = "User not found"),
    (status = 400, description = "Invalid ObjectId")
  ),
  tag = "users"
)]
pub async fn get_user_by_id(
  State(state): State<Arc<AppState>>,
  Path(id): Path<String>,
) -> Result<Json<UserResponseDto>, (StatusCode, String)> {
  let collection: Collection<User> = state.db.collection("users");
  let object_id = ObjectId::parse_str(&id).map_err(|_| (StatusCode::BAD_REQUEST, "Invalid ObjectId".to_string()))?;

  match collection.find_one(doc! { "_id": object_id }, None).await {
    Ok(Some(user)) => {
      Ok(Json(UserResponseDto {
        id: user.id.unwrap().to_string(),
        name: user.name,
        email: user.email
      }))
    },
    Ok(None) => Err((StatusCode::NOT_FOUND, "User not found".to_string())),
    Err(_) => Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch user".to_string()))
  }
}

#[utoipa::path(
  put,
  path = "/api/users/{id}",
  params(
    ( "id" = String, Path, description = "User ID")
  ),
  request_body = UpdateUserDto,
  responses(
    (status = 200, description = "User updated successfully", body = UserResponseDto),
    (status = 404, description = "User not found"),
    (status = 400, description = "Invalid ObjectId"),
    (status = 500, description = "Internal Server Error")
  ),
  tag = "users"
)]
pub async fn update_user(
  State(state): State<Arc<AppState>>,
  Path(id): Path<String>,
  Json(payload): Json<UpdateUserDto>
) -> Result<Json<UserResponseDto>, (StatusCode, String)> {
  let collection: Collection<User> = state.db.collection("users");
  let object_id = ObjectId::parse_str(&id).map_err(|_| (StatusCode::BAD_REQUEST, "Invalid ObjectId".to_string()))?;
  
  let existing_user = collection.find_one(doc! { "_id": object_id }, None).await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch user".to_string()))?
    .ok_or((StatusCode::NOT_FOUND, "User not found".to_string()))?;
  
  let update = doc! {
    "$set": {
      "name": payload.name.unwrap_or(existing_user.name),
      "email": payload.email.unwrap_or(existing_user.email),
    }
  };
  
  match collection.find_one_and_update(doc! { "_id": object_id }, update, None).await {
    Ok(Some(updated_user)) => Ok(Json(UserResponseDto {
      id: updated_user.id.unwrap().to_string(),
      name: updated_user.name,
      email: updated_user.email
    })),
    Ok(None) => Err((StatusCode::NOT_FOUND, "User not found".to_string())),
    Err(_) => Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to update user".to_string()))
  }
}

#[utoipa::path(
  delete,
  path = "/api/users/{id}",
  params(
    ( "id" = String, Path, description = "User ID")
  ),
  responses(
    (status = 204, description = "User deleted successfully"),
    (status = 400, description = "Invalid ObjectId"),
    (status = 404, description = "User not found"),
    (status = 500, description = "Internal Server Error")
  ),
  tag = "users"
)]
pub async fn delete_user(
  State(state): State<Arc<AppState>>,
  Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
  let collection: Collection<User> = state.db.collection("users");
  let object_id = ObjectId::parse_str(&id).map_err(|_| (StatusCode::BAD_REQUEST, "Invalid ObjectId".to_string()))?;
  match collection.delete_one(doc! { "_id": object_id }, None).await {
    Ok(result) if result.deleted_count == 1 => Ok(StatusCode::NO_CONTENT),
    Ok(_) => Err((StatusCode::NOT_FOUND, "User not found".to_string())),
    Err(_) => Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to delete user".to_string()))
  }
}