mod login;
mod redis_client;
mod register_user;
mod token;

use axum::{
    Router,
    routing::{get, post},
};

use migration::sea_orm::{self, DatabaseConnection};
use r2d2::Pool;
use tokio::net::TcpListener;

#[derive(Clone)]
pub struct AppState {
    redis: Pool<redis::Client>,
    db: DatabaseConnection,
}

#[tokio::main]
async fn main() {
    let addr = "0.0.0.0:3000";
    let listener = TcpListener::bind(addr).await.unwrap();

    // Redis connections pool
    // let client = redis::Client::open("redis://0.0.0.0/").unwrap();
    let client = redis::Client::open("redis://my-redis:6379").unwrap();
    let pool = r2d2::Pool::builder()
        .build(client)
        .expect("Error: Unable to connect with redis ...");

    let db_url = std::env::var("POSTGRES_URL")
        .unwrap_or("postgres://postgres:admin@my-db/temptor".to_string());
    let conn: DatabaseConnection = sea_orm::Database::connect(db_url).await.unwrap();

    let state = AppState {
        redis: pool,
        db: conn,
    };

    let router = Router::new()
        .route("/", get(ping))
        .route("/user", post(register_user::create_user))
        .route("/user/{id}", get(register_user::get_user))
        .route("/get-token", post(login::get_token))
        .with_state(state);

    println!("Server starts at: {addr}");

    axum::serve(listener, router).await.unwrap();
}

async fn ping() -> &'static str {
    "pong"
}
