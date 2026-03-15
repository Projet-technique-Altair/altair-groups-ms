use tower_http::cors::{Any, CorsLayer};

mod error;
mod models;
mod routes;
mod services;
mod state;

use crate::routes::init_routes;
use crate::state::AppState;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let state = AppState::new().await;

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = init_routes().with_state(state).layer(cors);

    let port = std::env::var("PORT").unwrap_or_else(|_| "3006".to_string());

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .expect("Failed to bind groups-ms port");

    println!("Groups MS running on http://localhost:{}", port);

    axum::serve(listener, app).await.unwrap();
}
