mod api;
mod middleware;
mod ws;

use porter_core::agents::AgentManager;
use porter_core::config::PorterConfig;
use porter_core::db::{self, Database};
use porter_core::integrations::IntegrationRegistry;
use porter_core::models::{Task, WsEvent};
use porter_integrations::register_builtin_integrations;
use sqlx::sqlite::SqlitePoolOptions;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::broadcast;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<PorterConfig>,
    pub db: Database,
    pub integration_registry: Arc<IntegrationRegistry>,
    pub agent_manager: Arc<AgentManager>,
    pub ws_tx: broadcast::Sender<WsEvent>,
    pub started_at: Instant,
}

impl AppState {
    pub fn broadcast_task_created(&self, task: &Task) {
        let _ = self.ws_tx.send(WsEvent::TaskCreated(task.clone()));
    }

    pub fn broadcast_task_updated(&self, task: &Task) {
        let _ = self.ws_tx.send(WsEvent::TaskUpdated(task.clone()));
    }

    pub fn broadcast_task_deleted(&self, id: &str) {
        let _ = self.ws_tx.send(WsEvent::TaskDeleted {
            id: id.to_string(),
        });
    }
}

pub async fn run_server(config: PorterConfig) -> anyhow::Result<()> {
    // Database setup
    let db_url = format!("sqlite:{}?mode=rwc", config.instance.db_path);
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&db_url)
        .await?;

    db::run_migrations(&pool).await?;
    let database = Database::new(pool);

    // Integration registry
    let mut registry = IntegrationRegistry::new();
    register_builtin_integrations(&mut registry, &config.integrations.enabled);

    // Agent manager (with MCP server configs)
    let agent_manager = AgentManager::new(
        database.clone(),
        config.agents.claude_binary.clone(),
        config.agents.max_concurrent_sessions,
        config.agents.default_model.clone(),
        config.agents.mcp.clone(),
    );

    // WebSocket broadcast channel
    let (ws_tx, _) = broadcast::channel::<WsEvent>(256);

    let state = AppState {
        config: Arc::new(config.clone()),
        db: database,
        integration_registry: Arc::new(registry),
        agent_manager: Arc::new(agent_manager),
        ws_tx,
        started_at: Instant::now(),
    };

    // Build router
    let app = axum::Router::new()
        .merge(api::router())
        .merge(ws::router())
        .layer(middleware::cors_layer())
        .layer(middleware::trace_layer())
        .with_state(state);

    let addr = format!("0.0.0.0:{}", config.instance.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!(
        "Porter server '{}' listening on {}",
        config.instance.name,
        addr
    );

    axum::serve(listener, app).await?;

    Ok(())
}
