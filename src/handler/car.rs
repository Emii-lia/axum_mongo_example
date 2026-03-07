use std::sync::Arc;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use futures_util::TryStreamExt;
use mongodb::bson::doc;
use mongodb::bson::oid::ObjectId;
use mongodb::Collection;
use crate::dto::car::{CarResponseDto, CreateCarDto, UpdateCarDto};
use crate::model::car::Car;
use crate::state::AppState;

#[utoipa::path(
  post,
  path = "/api/cars",
  request_body = CreateCarDto,
  responses(
    (status = 201, description = "Car Added successfully", body = CarResponseDto),
    (status = 404, description = "Car not found"),
    (status = 500, description = "Internal Server Error")
  ),
  tag = "cars"
)]
pub async fn create_car(
  State(state): State<Arc<AppState>>,
  Json(payload): Json<CreateCarDto>
) -> Result<Json<CarResponseDto>, (StatusCode, String)> {
  let collection = state.db.collection("cars");

  let new_car = Car {
    id: None,
    name: payload.name,
    owner_id: ObjectId::parse_str(&payload.owner_id).map_err(|_| (StatusCode::BAD_REQUEST, "Invalid ObjectId".to_string()))?
  };

  match collection.insert_one(new_car, None).await {
    Ok(result) => {
      let car_id = result.inserted_id.as_object_id().unwrap();
      let created_car = collection
        .find_one(doc! { "_id": car_id}, None)
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch car".to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Car not found".to_string()))?;
      Ok(Json(CarResponseDto {
        id: created_car.id.unwrap().to_string(),
        name: created_car.name,
        owner_id: created_car.owner_id.to_string()
      }))
    }
    Err(_) => {
      Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to create car".to_string()))
    }
  }
}

#[utoipa::path(
  get,
  path = "/api/cars",
  responses(
    (status = 200, description = "List of cars", body = Vec<CarResponseDto>),
    (status = 500, description = "Internal Server Error")
  ),
  tag = "cars"
)]
pub async fn list_cars(
  State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<CarResponseDto>>, (StatusCode, String)> {
  let collection = state.db.collection("cars");

  match collection.find(None, None).await {
    Ok(cursor) => {
      let docs: Vec<Car> = cursor.try_collect()
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch cars".to_string()))?;

      let cars: Vec<CarResponseDto> = docs.into_iter().map(|car| CarResponseDto {
        id: car.id.unwrap().to_string(),
        name: car.name,
        owner_id: car.owner_id.to_string()
      }).collect();

      Ok(Json(cars))
    }
    Err(_) => {
      Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch cars".to_string()))
    }
  }
}

#[utoipa::path(
  get,
  path = "/api/users/{user_id}/cars",
  params(
    ( "user_id" = String, Path, description = "User ID")
  ),
  responses(
    (status = 200, description = "List of cars by owner", body = Vec<CarResponseDto>),
    (status = 400, description = "Invalid ObjectId"),
    (status = 500, description = "Internal Server Error")
  ),
  tag = "cars"
)]
pub async fn list_cars_for_user(
  State(state): State<Arc<AppState>>,
  Path(user_id): Path<String>
) -> Result<Json<Vec<CarResponseDto>>, (StatusCode, String)> {
  let collection = state.db.collection("cars");

  let owner_id = ObjectId::parse_str(&user_id).map_err(|_| (StatusCode::BAD_REQUEST, "Invalid ObjectId".to_string()))?;

  match collection.find(doc! { "owner_id": owner_id }, None).await {
    Ok(cursor) => {
      let docs: Vec<Car> = cursor.try_collect()
        .await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch cars".to_string()))?;

      let cars: Vec<CarResponseDto> = docs.into_iter().map(|car| CarResponseDto {
        id: car.id.unwrap().to_string(),
        name: car.name,
        owner_id: car.owner_id.to_string()
      }).collect();

      Ok(Json(cars))
    }
    Err(_) => {
      Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch cars".to_string()))
    }
  }
}

#[utoipa::path(
  get,
  path = "/api/cars/{id}",
  params(
    ( "id" = String, Path, description = "Car ID")
  ),
  responses(
    (status = 200, description = "Car details", body = CarResponseDto),
    (status = 404, description = "Car not found"),
    (status = 400, description = "Invalid ObjectId")
  ),
  tag = "cars"
)]
pub async fn get_car_by_id(
  State(state): State<Arc<AppState>>,
  Path(id): Path<String>,
) -> Result<Json<CarResponseDto>, (StatusCode, String)> {
  let collection: Collection<Car> = state.db.collection("cars");
  let object_id = ObjectId::parse_str(&id).map_err(|_| (StatusCode::BAD_REQUEST, "Invalid ObjectId".to_string()))?;

  match collection.find_one(doc! { "_id": object_id}, None).await {
    Ok(Some(car)) => {
      Ok(Json(CarResponseDto {
        id: car.id.unwrap().to_string(),
        name: car.name,
        owner_id: car.owner_id.to_string()
      }))
    }
    Ok(None) => Err((StatusCode::NOT_FOUND, "Car not found".to_string())),
    Err(_) => Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch car".to_string()))
  }
}

#[utoipa::path(
  put,
  path = "/api/cars/{id}",
  params(
    ( "id" = String, Path, description = "Car ID")
  ),
  request_body = UpdateCarDto,
  responses(
    (status = 200, description = "Car updated successfully", body = CarResponseDto),
    (status = 404, description = "Car not found"),
    (status = 400, description = "Invalid ObjectId"),
    (status = 500, description = "Internal Server Error")
  ),
  tag = "cars"
)]
pub async fn update_car(
  State(state): State<Arc<AppState>>,
  Path(id): Path<String>,
  Json(payload): Json<UpdateCarDto>
) -> Result<Json<CarResponseDto>, (StatusCode, String)> {
  let collection: Collection<Car> = state.db.collection("cars");
  let object_id = ObjectId::parse_str(&id).map_err(|_| (StatusCode::BAD_REQUEST, "Invalid ObjectId".to_string()))?;

  let existing_car = collection.find_one(doc! { "_id": object_id }, None).await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch car".to_string()))?
    .ok_or((StatusCode::NOT_FOUND, "Car not found".to_string()))?;

  let update = doc! {
    "$set": {
      "name": payload.name.clone().unwrap_or(existing_car.name.clone()),
    }
  };

  match collection.update_one(doc! { "_id": object_id }, update, None).await {
    Ok(_) => Ok(Json(CarResponseDto {
      id: existing_car.id.unwrap().to_string(),
      name: payload.name.unwrap_or(existing_car.name),
      owner_id: existing_car.owner_id.to_string()
    })),
    Err(_) => Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to update car".to_string()))
  }
}

#[utoipa::path(
  delete,
  path = "/api/cars/{id}",
  params(
    ( "id" = String, Path, description = "Car ID")
  ),
  responses(
    (status = 204, description = "Car deleted successfully"),
    (status = 404, description = "Car not found"),
    (status = 400, description = "Invalid ObjectId"),
  ),
  tag = "cars"
)]
pub async fn delete_car(
  State(state): State<Arc<AppState>>,
  Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
  let collection: Collection<Car> = state.db.collection("cars");
  let object_id = ObjectId::parse_str(&id).map_err(|_| (StatusCode::BAD_REQUEST, "Invalid ObjectId".to_string()))?;
  match collection.delete_one(doc! { "_id": object_id }, None).await {
    Ok(_) => Ok(StatusCode::NO_CONTENT),
    Err(_) => Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to delete car".to_string()))
  }
}