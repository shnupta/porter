use crate::models::Notification;
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Configuration passed to a skill during initialization.
#[derive(Debug, Clone, Default)]
pub struct SkillConfig {
    pub values: HashMap<String, toml::Value>,
}

/// An action/command to be handled by a skill.
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

/// A capability that a skill exposes to Claude agents.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

/// The core skill trait that all skills must implement.
#[async_trait]
pub trait Skill: Send + Sync {
    /// Unique identifier for this skill.
    fn id(&self) -> &str;

    /// Human-readable name.
    fn name(&self) -> &str;

    /// Initialize the skill with its configuration.
    async fn init(&mut self, config: &SkillConfig) -> Result<()>;

    /// Handle an incoming action/command.
    async fn handle(&self, action: Action) -> Result<ActionResult>;

    /// Background tick - called periodically for polling-based skills.
    async fn tick(&self) -> Result<Vec<Notification>>;

    /// Capabilities this skill provides to Claude agents.
    fn capabilities(&self) -> Vec<Capability>;
}

/// Registry that manages all registered skills.
pub struct SkillRegistry {
    skills: HashMap<String, Arc<dyn Skill>>,
}

impl SkillRegistry {
    pub fn new() -> Self {
        Self {
            skills: HashMap::new(),
        }
    }

    pub fn register(&mut self, skill: Arc<dyn Skill>) {
        self.skills.insert(skill.id().to_string(), skill);
    }

    pub fn get(&self, id: &str) -> Option<&Arc<dyn Skill>> {
        self.skills.get(id)
    }

    pub fn list(&self) -> Vec<&Arc<dyn Skill>> {
        self.skills.values().collect()
    }

    pub fn ids(&self) -> Vec<String> {
        self.skills.keys().cloned().collect()
    }
}

impl Default for SkillRegistry {
    fn default() -> Self {
        Self::new()
    }
}
