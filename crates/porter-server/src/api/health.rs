use crate::AppState;
use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use porter_core::models::ServerStatus;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/health", get(health))
        .route("/api/status", get(status))
}

async fn health() -> &'static str {
    "ok"
}

async fn status(State(state): State<AppState>) -> Json<ServerStatus> {
    let active_sessions = state
        .agent_manager
        .list_sessions(Some("running"))
        .await
        .map(|s| s.len())
        .unwrap_or(0);

    let pending_tasks = state
        .db
        .count_tasks_by_status("pending")
        .await
        .unwrap_or(0) as usize;

    let uptime = state.started_at.elapsed().as_secs();

    Json(ServerStatus {
        instance_name: state.config.instance.name.clone(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: uptime,
        active_skills: state.skill_registry.ids(),
        active_agent_sessions: active_sessions,
        pending_tasks,
    })
}
