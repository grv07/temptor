use crate::AppState;
use crate::token::generate_token;
use ::entity::prelude::Users as UsersEntity;
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::response::IntoResponse;
use axum::{Json, extract::State, http::StatusCode};
use entity::users;
use migration::Condition;
use migration::sea_orm::ColumnTrait;
use migration::sea_orm::{EntityTrait, QueryFilter};
use serde_json::json;

pub async fn get_token(
    State(state): State<AppState>,
    Json(user): Json<users::Model>,
) -> impl IntoResponse {
    if !is_valid(&user, state).await {
        let res = json!({
            "token": "",
            "message": "fail"
        });
        return (StatusCode::UNAUTHORIZED, Json(res));
    }

    if let Ok(token) = generate_token(user.name, "1234".to_string()) {
        let res = json!({
            "token": token,
            "message": "success"
        });
        (StatusCode::OK, Json(res))
    } else {
        let res = json!({
            "token": "",
            "message": "fail"
        });
        (StatusCode::INTERNAL_SERVER_ERROR, Json(res))
    }
}

async fn is_valid(user: &users::Model, state: AppState) -> bool {
    let find_user = UsersEntity::find()
        .filter(
            Condition::any()
                .add(users::Column::Name.eq(user.name.clone()))
                .add(users::Column::Email.eq(user.email.clone())),
        )
        .one(&state.db)
        .await
        .unwrap()
        .unwrap();

    let argon2 = Argon2::default();

    let hash_pass = PasswordHash::new(&find_user.password).unwrap();

    if let Ok(_) = argon2.verify_password(user.password.as_bytes(), &hash_pass) {
        println!("Password verified ... ");
        true
    } else {
        false
    }
}
