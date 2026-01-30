mod api;
mod middleware;
mod ws;

use porter_core::agents::{AgentEvent, AgentManager};
use porter_core::config::PorterConfig;
use porter_core::db::{self, Database};
use porter_core::integrations::IntegrationRegistry;
use porter_core::models::{Task, WsEvent};
use porter_integrations::register_builtin_integrations;
use sqlx::sqlite::SqlitePoolOptions;
use std::sync::Arc;
use std::time::{Duration, Instant};
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
    register_builtin_integrations(&mut registry, &config.integrations, database.clone()).await;

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

    // Collect tick integrations before moving registry into Arc
    let tick_integrations = registry.tick_integrations();

    let state = AppState {
        config: Arc::new(config.clone()),
        db: database.clone(),
        integration_registry: Arc::new(registry),
        agent_manager: Arc::new(agent_manager),
        ws_tx: ws_tx.clone(),
        started_at: Instant::now(),
    };

    // Spawn background tick tasks for integrations that have a configured interval
    for (integration, interval_secs) in tick_integrations {
        let db = database.clone();
        let tx = ws_tx.clone();
        let id = integration.id().to_string();
        tracing::info!(integration = %id, interval_secs, "Spawning tick task");

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(interval_secs));
            loop {
                interval.tick().await;
                tracing::debug!(integration = %id, "Running tick");
                match integration.tick().await {
                    Ok(notifications) => {
                        for notification in notifications {
                            if let Err(e) = db
                                .create_notification(
                                    &notification.notification_type,
                                    &notification.message,
                                    notification.integration_id.as_deref(),
                                )
                                .await
                            {
                                tracing::error!(integration = %id, error = %e, "Failed to persist tick notification");
                            }
                            let _ = tx.send(WsEvent::Notification(notification));
                        }
                    }
                    Err(e) => {
                        tracing::error!(integration = %id, error = %e, "Tick failed");
                    }
                }
            }
        });
    }

    // Forward agent events to the WebSocket broadcast channel
    {
        let mut agent_rx = state.agent_manager.subscribe();
        let ws_tx = state.ws_tx.clone();
        tokio::spawn(async move {
            while let Ok(event) = agent_rx.recv().await {
                let ws_event = match event {
                    AgentEvent::Output { session_id, content } => {
                        WsEvent::AgentOutput { session_id, content }
                    }
                    AgentEvent::StatusChanged { session_id, status } => {
                        WsEvent::AgentStatusChanged { session_id, status }
                    }
                };
                let _ = ws_tx.send(ws_event);
            }
        });
    }

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
