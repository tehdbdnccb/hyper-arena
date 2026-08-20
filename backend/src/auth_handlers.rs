use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{auth::create_jwt, AppState};

#[derive(Deserialize)]
pub struct AuthPayload {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub user_id: Uuid,
    pub username: String,
}

fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
}

fn verify_password(password: &str, password_hash: &str) -> bool {
    let parsed_hash = match PasswordHash::new(password_hash) {
        Ok(h) => h,
        Err(_) => return false,
    };
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
}

pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<AuthPayload>,
) -> Result<Json<AuthResponse>, (StatusCode, String)> {
    if payload.username.trim().is_empty() || payload.password.len() < 6 {
        return Err((
            StatusCode::BAD_REQUEST,
            "Username required, password min 6 chars".to_string(),
        ));
    }

    let password_hash = hash_password(&payload.password)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Hashing failed".to_string()))?;

    let record = sqlx::query_as::<_, (Uuid, String)>(
        r#"INSERT INTO players (username, password_hash) VALUES ($1, $2) RETURNING id, username"#,
    )
    .bind(&payload.username)
    .bind(&password_hash)
    .fetch_one(&state.db)
    .await
    .map_err(|err| {
        if let sqlx::Error::Database(db_err) = &err {
            if db_err.is_unique_violation() {
                return (StatusCode::CONFLICT, "Username taken".to_string());
            }
        }
        (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string())
    })?;

    let token = create_jwt(record.0, &record.1)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Token failed".to_string()))?;

    Ok(Json(AuthResponse {
        token,
        user_id: record.0,
        username: record.1,
    }))
}

pub async fn login(
    State(state): State<AppState>,
    Json(payload): Json<AuthPayload>,
) -> Result<Json<AuthResponse>, (StatusCode, String)> {
    let user = sqlx::query_as::<_, (Uuid, String, String)>(
        r#"SELECT id, username, password_hash FROM players WHERE username = $1"#,
    )
    .bind(&payload.username)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string()))?
    .ok_or((StatusCode::UNAUTHORIZED, "Invalid credentials".to_string()))?;

    if !verify_password(&payload.password, &user.2) {
        return Err((StatusCode::UNAUTHORIZED, "Invalid credentials".to_string()));
    }

    let token = create_jwt(user.0, &user.1)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Token failed".to_string()))?;

    Ok(Json(AuthResponse {
        token,
        user_id: user.0,
        username: user.1,
    }))
}

