use crate::AppState;
use ::entity::prelude::Users as UsersEntity;
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::{Json, extract::State, http::StatusCode};
use entity::users;
use migration::Condition;
use migration::sea_orm::ColumnTrait;
use migration::sea_orm::{EntityTrait, QueryFilter};

pub async fn login(State(state): State<AppState>, Json(user): Json<users::Model>) -> StatusCode {
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
        StatusCode::OK
    } else {
        StatusCode::UNAUTHORIZED
    }
}
