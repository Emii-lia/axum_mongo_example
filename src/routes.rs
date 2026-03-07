use std::sync::Arc;
use axum::Router;
use axum::routing::{get, post};
use utoipa::OpenApi;
use crate::dto::car::{UpdateCarDto, CreateCarDto, CarResponseDto};
use crate::dto::user::{CreateUserDto, UpdateUserDto, UserResponseDto};
use crate::handler::car::{create_car, delete_car, get_car_by_id, list_cars, list_cars_for_user, update_car};
use crate::handler::health::health;
use crate::handler::openapi::{serve_api_docs, serve_swagger};
use crate::handler::user::{create_user, delete_user, get_user_by_id, list_users, update_user};
use crate::state::AppState;

#[derive(OpenApi)]
#[openapi(
  paths(
    crate::handler::user::list_users,
    crate::handler::user::create_user,
    crate::handler::user::get_user_by_id,
    crate::handler::user::update_user,
    crate::handler::user::delete_user,
    
    crate::handler::car::list_cars,
    crate::handler::car::create_car,
    crate::handler::car::get_car_by_id,
    crate::handler::car::update_car,
    crate::handler::car::delete_car,
    crate::handler::car::list_cars_for_user,
  
    crate::handler::health::health
  ),
  components(schemas(
    UserResponseDto,
    CreateUserDto,
    UpdateUserDto,
    CarResponseDto,
    CreateCarDto,
    UpdateCarDto,
  )),
  tags(
    ( name = "users", description = "User related operations"),
    ( name = "cars", description = "Car related operations"),
    ( name = "health", description = "Health check")
  )
)]
pub struct ApiDocs;

pub fn create_router(state: Arc<AppState>) -> Router {
  let user_routes = Router::new()
    .route("/api/users", post(create_user).get(list_users))
    .route("/api/users/{id}", get(get_user_by_id).put(update_user).delete(delete_user));
  
  let car_routes = Router::new()
    .route("/api/cars", post(create_car).get(list_cars))
    .route("/api/users/{user_id}/cars", get(list_cars_for_user))
    .route("/api/cars/{id}", get(get_car_by_id).put(update_car).delete(delete_car));

  Router::new()
    .route("/health", get(health))
    .route("/api-docs/openapi.json", get(serve_api_docs))
    .route("/swagger-ui", get(serve_swagger))
    .merge(user_routes)
    .merge(car_routes)
    .with_state(state)
}