use crate::AppState;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use porter_core::agents::SessionOptions;
use porter_core::models::{AgentMessage, AgentSession};
use serde::Deserialize;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/agents", get(list_sessions).post(start_session))
        .route("/api/agents/{id}", get(get_session))
        .route(
            "/api/agents/{id}/messages",
            get(get_messages).post(send_message),
        )
}

#[derive(Deserialize)]
struct SessionQuery {
    status: Option<String>,
}

#[derive(Deserialize)]
struct StartSessionRequest {
    prompt: String,
    directory: Option<String>,
    #[serde(default)]
    dangerously_skip_permissions: bool,
}

#[derive(Deserialize)]
struct SendMessageRequest {
    content: String,
}

async fn list_sessions(
    State(state): State<AppState>,
    Query(query): Query<SessionQuery>,
) -> Result<Json<Vec<AgentSession>>, StatusCode> {
    let sessions = state
        .agent_manager
        .list_sessions(query.status.as_deref())
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to list agent sessions");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    Ok(Json(sessions))
}

async fn start_session(
    State(state): State<AppState>,
    Json(input): Json<StartSessionRequest>,
) -> Result<(StatusCode, Json<AgentSession>), StatusCode> {
    let opts = SessionOptions {
        working_directory: input.directory,
        dangerously_skip_permissions: input.dangerously_skip_permissions,
    };

    let session = state
        .agent_manager
        .start_session(&input.prompt, opts)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to start agent session");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
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

async fn send_message(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(input): Json<SendMessageRequest>,
) -> Result<(StatusCode, Json<AgentMessage>), StatusCode> {
    // Verify session exists and has a claude_session_id
    let session = state
        .agent_manager
        .get_session(&id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    if session.claude_session_id.is_none() {
        return Err(StatusCode::BAD_REQUEST);
    }

    // send_message stores the user message and spawns the resume task
    state
        .agent_manager
        .send_message(&id, &input.content)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Fetch the just-stored user message to return it
    let messages = state
        .db
        .get_agent_messages(&id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let user_msg = messages
        .into_iter()
        .rev()
        .find(|m| m.role == "user")
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok((StatusCode::OK, Json(user_msg)))
}
