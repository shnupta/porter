use crate::AppState;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use porter_core::models::{AgentMessage, AgentSession};
use serde::Deserialize;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/agents", get(list_sessions).post(start_session))
        .route("/api/agents/{id}", get(get_session))
        .route("/api/agents/{id}/messages", get(get_messages))
}

#[derive(Deserialize)]
struct SessionQuery {
    status: Option<String>,
}

#[derive(Deserialize)]
struct StartSessionRequest {
    prompt: String,
}

async fn list_sessions(
    State(state): State<AppState>,
    Query(query): Query<SessionQuery>,
) -> Result<Json<Vec<AgentSession>>, StatusCode> {
    let sessions = state
        .agent_manager
        .list_sessions(query.status.as_deref())
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(sessions))
}

async fn start_session(
    State(state): State<AppState>,
    Json(input): Json<StartSessionRequest>,
) -> Result<(StatusCode, Json<AgentSession>), StatusCode> {
    let session = state
        .agent_manager
        .start_session(&input.prompt)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((StatusCode::CREATED, Json(session)))
}

async fn get_session(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<AgentSession>, StatusCode> {
    state
        .agent_manager
        .get_session(&id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

async fn get_messages(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Vec<AgentMessage>>, StatusCode> {
    let messages = state
        .db
        .get_agent_messages(&id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(messages))
}
