mod redis_client;

// use ::entity::users::{self as UsersEntity};
use argon2::password_hash::Salt;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};

use ::entity::prelude::Users as UsersEntity;
use ::entity::users;
use base64::Engine;
use migration::sea_orm::{self, ActiveValue, DatabaseConnection};
use r2d2::Pool;
use redis::Commands;
use sea_orm::entity::prelude::*;
use sea_orm::*;

use serde_json;
use tokio::net::TcpListener;

#[derive(Clone)]
struct AppState {
    redis: Pool<redis::Client>,
    db: DatabaseConnection,
}

#[tokio::main]
async fn main() {
    let addr = "0.0.0.0:3000";
    let listener = TcpListener::bind(addr).await.unwrap();

    // Redis connections pool
    let client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let pool = r2d2::Pool::builder()
        .build(client)
        .expect("Error: Unable to connect with redis ...");

    let conn: DatabaseConnection =
        // postgres://postgres:admin@localhost/temptor
        sea_orm::Database::connect("postgres://postgres:admin@localhost/temptor")
            .await
            .unwrap();

    let state = AppState {
        redis: pool,
        db: conn,
    };

    let router = Router::new()
        .route("/", get(ping))
        .route("/user", post(create_user))
        .route("/user/{id}", get(get_user))
        .route("/login", post(login))
        .with_state(state);

    println!("Server starts at: {addr}");

    axum::serve(listener, router).await.unwrap();
}

async fn ping() -> &'static str {
    "pong"
}

async fn get_user(Path(id): Path<String>, State(state): State<AppState>) -> impl IntoResponse {
    let mut rd = state.redis.get().unwrap();
    let user_id = format!("user:{id}");
    let user: String = rd.hget("users", user_id).unwrap();

    let data: users::Model = serde_json::from_str(user.as_str()).unwrap();

    Json(data)
}

async fn create_user(
    State(state): State<AppState>,
    Json(user): Json<users::Model>,
) -> Result<(StatusCode, Json<String>), StatusCode> {
    println!("Create user");
    let mut rd = state.redis.get().unwrap();

    let argon2 = Argon2::default();

    let b_salt = base64::engine::general_purpose::STANDARD.encode(format!("{}DEF", user.name));
    let salt_hash = Salt::from_b64(&b_salt).expect("Salt from b64");

    let hash_pass = argon2
        .hash_password(user.password.as_bytes(), salt_hash)
        .expect("Password is not valid")
        .to_string();

    println!("Create user: {hash_pass:?}");

    let user = users::ActiveModel {
        id: NotSet,
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

async fn login(State(state): State<AppState>, Json(user): Json<users::Model>) -> StatusCode {
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
