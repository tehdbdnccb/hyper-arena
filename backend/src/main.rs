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
use tower_http::cors::{Any, CorsLayer};

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

    let cors = CorsLayer::new()
        // Allow any origin (or you can restrict this to your Vercel URL later)
        .allow_origin(Any)
        // Explicitly allow the methods the browser needs
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        // Explicitly allow the headers your frontend is sending
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE]);

    let app = Router::new()
        .route("/api/auth/register", post(register))
        .route("/api/auth/login", post(login))
        .route("/api/player/profile", get(get_profile))
        .route("/ws", get(ws_handler))
        .layer(cors)
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 4000));
    println!("🚀 Server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

