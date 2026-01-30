use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
pub struct PorterConfig {
    pub instance: InstanceConfig,
    #[serde(default)]
    pub integrations: IntegrationsConfig,
    #[serde(default)]
    pub agents: AgentsConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct InstanceConfig {
    pub name: String,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_db_path")]
    pub db_path: String,
}

fn default_port() -> u16 {
    3100
}

fn default_db_path() -> String {
    "porter.db".to_string()
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct IntegrationsConfig {
    #[serde(default)]
    pub enabled: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AgentsConfig {
    #[serde(default = "default_claude_binary")]
    pub claude_binary: String,
    #[serde(default = "default_max_sessions")]
    pub max_concurrent_sessions: usize,
    #[serde(default = "default_model")]
    pub default_model: String,
    /// MCP servers available to Claude agent sessions.
    #[serde(default)]
    pub mcp: HashMap<String, McpServerConfig>,
}

impl Default for AgentsConfig {
    fn default() -> Self {
        Self {
            claude_binary: default_claude_binary(),
            max_concurrent_sessions: default_max_sessions(),
            default_model: default_model(),
            mcp: HashMap::new(),
        }
    }
}

fn default_claude_binary() -> String {
    "claude".to_string()
}

fn default_max_sessions() -> usize {
    5
}

fn default_model() -> String {
    "opus".to_string()
}

/// Configuration for an MCP server that Claude can use during agent sessions.
#[derive(Debug, Clone, Deserialize)]
pub struct McpServerConfig {
    /// The command to run (e.g. "npx", "node", "python").
    pub command: String,
    /// Arguments to the command.
    #[serde(default)]
    pub args: Vec<String>,
    /// Environment variables. Values prefixed with "env:" are read from
    /// the process environment at runtime.
    #[serde(default)]
    pub env: HashMap<String, String>,
}

impl PorterConfig {
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: PorterConfig = toml::from_str(&content)?;
        Ok(config)
    }
}
