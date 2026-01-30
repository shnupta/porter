use crate::AppState;
use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use porter_core::models::SkillInfo;

pub fn router() -> Router<AppState> {
    Router::new().route("/api/skills", get(list_skills))
}

async fn list_skills(State(state): State<AppState>) -> Json<Vec<SkillInfo>> {
    let skills: Vec<SkillInfo> = state
        .skill_registry
        .list()
        .iter()
        .map(|s| SkillInfo {
            id: s.id().to_string(),
            name: s.name().to_string(),
            enabled: true,
            capabilities: s.capabilities().iter().map(|c| c.name.clone()).collect(),
        })
        .collect();

    Json(skills)
}
