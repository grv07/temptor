use crate::AppState;

use argon2::{Argon2, PasswordHasher, password_hash::Salt};
use axum::extract::Path;
use axum::response::IntoResponse;
use axum::{Json, extract::State, http::StatusCode};
use base64::Engine;
use entity::users;
use migration::sea_orm::{ActiveModelTrait, ActiveValue, TryIntoModel};
use redis::Commands;

pub async fn create_user(
    State(state): State<AppState>,
    Json(user): Json<users::Model>,
) -> Result<(StatusCode, Json<String>), StatusCode> {
    println!("Create user");

    let argon2 = Argon2::default();

    let b_salt = base64::engine::general_purpose::STANDARD.encode(format!("{}DEF", user.name));
    let salt_hash = Salt::from_b64(&b_salt).expect("Salt from b64");

    let hash_pass = argon2
        .hash_password(user.password.as_bytes(), salt_hash)
        .expect("Password is not valid")
        .to_string();

    println!("Create user: {hash_pass:?}");

    let user = users::ActiveModel {
        id: ActiveValue::NotSet,
        name: ActiveValue::Set(user.name),
        email: ActiveValue::Set(user.email),
        password: ActiveValue::Set(hash_pass),
    };

    if let Ok(user) = user.save(&state.db).await {
        let user = user.try_into_model().unwrap();

        let data = serde_json::to_string_pretty(&user).unwrap();

        let id = format!("user:{}", user.id);
        println!("id: {id:?}");

        Ok((StatusCode::OK, Json(data)))
    } else {
        Err(StatusCode::EXPECTATION_FAILED)
    }
}

pub async fn get_user(Path(id): Path<String>, State(state): State<AppState>) -> impl IntoResponse {
    let mut rd = state.redis.get().unwrap();
    let user_id = format!("user:{id}");
    let user: String = rd.hget("users", user_id).unwrap();

    let data: users::Model = serde_json::from_str(user.as_str()).unwrap();

    Json(data)
}
