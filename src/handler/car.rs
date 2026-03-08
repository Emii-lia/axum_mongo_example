use std::sync::Arc;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use futures_util::TryStreamExt;
use mongodb::bson::{doc, Document};
use mongodb::bson::oid::ObjectId;
use mongodb::{bson, Collection};
use crate::dto::car::{CarDetailsDto, CarResponseDto, CreateCarDto, UpdateCarDto};
use crate::dto::user::UserResponseDto;
use crate::model::car::Car;
use crate::model::user::User;
use crate::state::AppState;
use crate::utils::mongo::{match_filter, populate, project};

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
  let car_collection = state.db.collection("cars");
  let user_collection: Collection<User> = state.db.collection("users");

  let new_car = Car {
    id: None,
    name: payload.name,
    owner_id: ObjectId::parse_str(&payload.owner_id).map_err(|_| (StatusCode::BAD_REQUEST, "Invalid ObjectId".to_string()))?
  };

  match car_collection.insert_one(new_car, None).await {
    Ok(result) => {
      let car_id = result.inserted_id.as_object_id().unwrap();
      let car = car_collection.find_one(doc! { "_id": car_id }, None).await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch car".to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Car not found".to_string()))?;
      let owner= user_collection.find_one(doc! { "_id": car.owner_id }, None).await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch owner".to_string()))?
        .ok_or((StatusCode::NOT_FOUND, "Owner not found".to_string()))?;

      Ok(Json(CarResponseDto {
        id: car.id.unwrap().to_string(),
        name: car.name,
        owner: Some(UserResponseDto {
          id: owner.id.unwrap().to_string(),
          name: owner.name,
          email: owner.email
        })
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
  let collection = state.db.collection::<Document>("cars");

  let mut pipeline = vec![];

  pipeline.extend(populate("users", "owner_id", "_id", "owner"));
  pipeline.push(project(vec![
    ("id", 1),
    ("name", 1),
    ("owner._id", 1),
    ("owner.name", 1),
    ("owner.email", 1)
  ]));

  let mut cursor = collection.aggregate(pipeline, None).await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch cars".to_string()))?;

  let mut cars: Vec<CarResponseDto> = Vec::new();

  while let Some(result) = cursor.try_next().await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch cars".to_string()))? {
    let car: CarDetailsDto = bson::from_document(result)
      .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to parse car data".to_string()))?;

    cars.push(CarResponseDto {
      id: car.id.to_string(),
      name: car.name,
      owner: car.owner.map(|o| UserResponseDto {
        id: o.id.unwrap().to_string(),
        name: o.name,
        email: o.email
      })
    });
  };

  Ok(Json(cars))
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
  let collection = state.db.collection::<Document>("cars");

  let owner_id = ObjectId::parse_str(&user_id).map_err(|_| (StatusCode::BAD_REQUEST, "Invalid ObjectId".to_string()))?;

  let mut pipeline = vec![];
  pipeline.push(match_filter(doc! { "owner_id": owner_id }));
  pipeline.extend(populate("users", "owner_id", "_id", "owner"));
  pipeline.push(project(vec![
    ("id", 1),
    ("name", 1),
    ("owner._id", 1),
    ("owner.name", 1),
    ("owner.email", 1)
  ]));

  let mut cursor = collection.aggregate(pipeline, None).await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch cars".to_string()))?;

  let mut cars: Vec<CarResponseDto> = Vec::new();

  while let Some(result) = cursor.try_next().await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch cars".to_string()))? {

    let car: CarDetailsDto = bson::from_document(result)
      .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to parse car data".to_string()))?;

    cars.push(CarResponseDto {
      id: car.id.to_string(),
      name: car.name,
      owner: car.owner.map(|o| UserResponseDto {
        id: o.id.unwrap().to_string(),
        name: o.name,
        email: o.email
      })
    });
  };

  Ok(Json(cars))
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
  let car_collection: Collection<Car> = state.db.collection("cars");
  let user_collection: Collection<Document> = state.db.collection("users");
  let object_id = ObjectId::parse_str(&id).map_err(|_| (StatusCode::BAD_REQUEST, "Invalid ObjectId".to_string()))?;

  match car_collection.find_one(doc! { "_id": object_id }, None).await {
    Ok(Some(car)) => {
      let owner = user_collection.find_one(doc! { "_id": car.owner_id }, None).await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch owner".to_string()))?;

      Ok(Json(CarResponseDto {
        id: car.id.unwrap().to_string(),
        name: car.name,
        owner: owner.map(|o| UserResponseDto {
          id: o.get_object_id("_id").unwrap().to_string(),
          name: o.get_str("name").unwrap().to_string(),
          email: o.get_str("email").unwrap().to_string()
        })
      }))
    },
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
  let car_collection: Collection<Car> = state.db.collection("cars");
  let user_collection: Collection<Document> = state.db.collection("users");
  let object_id = ObjectId::parse_str(&id).map_err(|_| (StatusCode::BAD_REQUEST, "Invalid ObjectId".to_string()))?;

  let existing_car = car_collection.find_one(doc! { "_id": object_id }, None).await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch car".to_string()))?
    .ok_or((StatusCode::NOT_FOUND, "Car not found".to_string()))?;

  let update = doc! {
    "$set": {
      "name": payload.name.clone().unwrap_or(existing_car.name.clone()),
    }
  };

  match car_collection.update_one(doc! { "_id": object_id }, update, None).await {
    Ok(_) => {
      let owner = user_collection.find_one(doc! { "_id": existing_car.owner_id }, None).await
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch owner".to_string()))?;

      Ok(Json(CarResponseDto {
        id: existing_car.id.unwrap().to_string(),
        name: payload.name.unwrap_or(existing_car.name),
        owner: owner.map(|o| UserResponseDto {
          id: o.get_object_id("_id").unwrap().to_string(),
          name: o.get_str("name").unwrap().to_string(),
          email: o.get_str("email").unwrap().to_string()
        })
      }))
    },
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