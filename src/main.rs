mod redis_client;

use axum::{Json, Router, extract::State, http::StatusCode, response::IntoResponse, routing::get};
use r2d2::Pool;
use redis::{Commands, RedisResult};
use serde::{Deserialize, Serialize};
use serde_json::{self, Value};
use tokio::net::TcpListener;

#[derive(Clone)]
struct AppState {
    redis_pool: Pool<redis::Client>,
}

#[tokio::main]
async fn main() {
    let addr = "0.0.0.0:3000";
    let listener = TcpListener::bind(addr).await.unwrap();

    // Redis connections pool
    let client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let pool = r2d2::Pool::builder().build(client).unwrap();

    let state = AppState { redis_pool: pool };

    let router = Router::new()
        .route("/", get(ping))
        .route("/user", get(get_user).post(create_user))
        .with_state(state);

    println!("Server starts at: {addr}");

    axum::serve(listener, router).await.unwrap();
}

async fn ping() -> &'static str {
    "pong"
}

#[derive(Deserialize, Serialize, Debug)]
struct User {
    id: u64,
    name: String,
    email: String,
}

async fn get_user(State(state): State<AppState>) -> impl IntoResponse {
    let mut rd = state.redis_pool.get().unwrap();
    let user: String = rd.hget("users", "user:1").unwrap();

    let data: User = serde_json::from_str(user.as_str()).unwrap();

    Json(data)
}

async fn create_user(
    State(state): State<AppState>,
    Json(user): Json<User>,
) -> (StatusCode, Json<User>) {
    // TODO:
    // db.create_user(user);

    let id = format!("user:{}", user.id);

    let mut rd = state.redis_pool.get().unwrap();
    let _: () = rd.set("K", 34).unwrap();
    let data = serde_json::to_string(&user).unwrap();
    let _: () = rd.hset("users", id, data).unwrap();

    (StatusCode::OK, Json::from(user))
}
