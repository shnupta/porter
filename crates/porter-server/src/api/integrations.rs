use crate::AppState;
use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use porter_core::models::{IntegrationInfo, McpServerInfo};
use serde::Serialize;

#[derive(Serialize)]
struct IntegrationsResponse {
    integrations: Vec<IntegrationInfo>,
    mcp_servers: Vec<McpServerInfo>,
}

pub fn router() -> Router<AppState> {
    Router::new().route("/api/integrations", get(list_integrations))
}

async fn list_integrations(State(state): State<AppState>) -> Json<IntegrationsResponse> {
    let integrations: Vec<IntegrationInfo> = state
        .integration_registry
        .list()
        .iter()
        .map(|i| IntegrationInfo {
            id: i.id().to_string(),
            name: i.name().to_string(),
            enabled: true,
            capabilities: i.capabilities().iter().map(|c| c.name.clone()).collect(),
        })
        .collect();

    let mcp_servers: Vec<McpServerInfo> = state
        .config
        .agents
        .mcp
        .iter()
        .map(|(name, config)| McpServerInfo {
            name: name.clone(),
            command: config.command.clone(),
        })
        .collect();

    Json(IntegrationsResponse {
        integrations,
        mcp_servers,
    })
}
