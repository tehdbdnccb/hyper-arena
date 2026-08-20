mod auth;
mod auth_handlers;
mod protected_handlers;
mod ws_handler;

use axum::{
    routing::{get, post},
    Router,
    http::{Method, header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE}},
};
use sqlx::PgPool;
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;

use auth_handlers::{login, register};
use protected_handlers::get_profile;
use ws_handler::ws_handler;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/arena".to_string());
        
    let db_pool = PgPool::connect(&db_url).await?;
    
    // Run migrations on startup
    println!("🔄 Running database migrations...");
    sqlx::migrate!("./migrations")
        .run(&db_pool)
        .await?;
    println!("✅ Migrations completed successfully");
    
    let state = AppState { db: db_pool };

    // Build CORS layer with explicit origin, methods, and headers
    let cors = CorsLayer::permissive()
        .allow_credentials(true)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE]);

    // Create inner router with all API routes
    let api_routes = Router::new()
        .route("/auth/register", post(register))
        .route("/auth/login", post(login))
        .route("/player/profile", get(get_profile))
        .route("/ws", get(ws_handler))
        .with_state(state);

    // Build main app with CORS middleware wrapping everything
    let app = Router::new()
        .nest("/api", api_routes)
        .layer(cors);

    let addr = SocketAddr::from(([0, 0, 0, 0], 4000));
    println!("🚀 Server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

