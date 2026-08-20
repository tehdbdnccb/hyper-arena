use axum::{extract::State, http::StatusCode, Json};
use serde_json::{json, Value};
use crate::{auth::Claims, AppState};
use chrono::{DateTime, Utc};
use uuid::Uuid;

pub async fn get_profile(
    claims: Claims,
    State(state): State<AppState>,
) -> Result<Json<Value>, StatusCode> {
    let profile = sqlx::query_as::<_, (String, DateTime<Utc>)>(
        r#"SELECT username, created_at FROM players WHERE id = $1"#,
    )
    .bind(claims.sub)
    .fetch_optional(&state.db)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(json!({
        "id": claims.sub,
        "username": profile.0,
        "joined_at": profile.1
    })))
}

