use crate::models::Notification;
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Configuration passed to an integration during initialization.
#[derive(Debug, Clone, Default)]
pub struct IntegrationConfig {
    pub values: HashMap<String, toml::Value>,
}

/// An action/command to be handled by an integration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    pub name: String,
    pub params: serde_json::Value,
}

/// Result of handling an action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionResult {
    pub success: bool,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

/// A capability that an integration exposes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

/// Core trait for built-in integrations that need background processing
/// or tight coupling with Porter (e.g. tasks, notifications).
///
/// External API integrations (restaurants, flights, Slack, etc.) should
/// use MCP servers instead — configured in TOML under `[agents.mcp]`.
#[async_trait]
pub trait Integration: Send + Sync {
    /// Unique identifier for this integration.
    fn id(&self) -> &str;

    /// Human-readable name.
    fn name(&self) -> &str;

    /// Initialize the integration with its configuration.
    async fn init(&mut self, config: &IntegrationConfig) -> Result<()>;

    /// Handle an incoming action/command.
    async fn handle(&self, action: Action) -> Result<ActionResult>;

    /// Background tick - called periodically for polling-based integrations.
    async fn tick(&self) -> Result<Vec<Notification>>;

    /// Capabilities this integration provides.
    fn capabilities(&self) -> Vec<Capability>;
}

/// Registry that manages all registered integrations.
pub struct IntegrationRegistry {
    integrations: HashMap<String, Arc<dyn Integration>>,
}

impl IntegrationRegistry {
    pub fn new() -> Self {
        Self {
            integrations: HashMap::new(),
        }
    }

    pub fn register(&mut self, integration: Arc<dyn Integration>) {
        self.integrations
            .insert(integration.id().to_string(), integration);
    }

    pub fn get(&self, id: &str) -> Option<&Arc<dyn Integration>> {
        self.integrations.get(id)
    }

    pub fn list(&self) -> Vec<&Arc<dyn Integration>> {
        self.integrations.values().collect()
    }

    pub fn ids(&self) -> Vec<String> {
        self.integrations.keys().cloned().collect()
    }
}

impl Default for IntegrationRegistry {
    fn default() -> Self {
        Self::new()
    }
}
