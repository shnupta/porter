use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Deserialize)]
pub struct PorterConfig {
    pub instance: InstanceConfig,
    #[serde(default)]
    pub skills: SkillsConfig,
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
pub struct SkillsConfig {
    #[serde(default)]
    pub enabled: Vec<String>,
    #[serde(flatten)]
    pub skill_configs: HashMap<String, toml::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AgentsConfig {
    #[serde(default = "default_claude_binary")]
    pub claude_binary: String,
    #[serde(default = "default_max_sessions")]
    pub max_concurrent_sessions: usize,
    #[serde(default = "default_model")]
    pub default_model: String,
    #[serde(default)]
    pub skills: AgentSkillsConfig,
}

impl Default for AgentsConfig {
    fn default() -> Self {
        Self {
            claude_binary: default_claude_binary(),
            max_concurrent_sessions: default_max_sessions(),
            default_model: default_model(),
            skills: AgentSkillsConfig::default(),
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

#[derive(Debug, Clone, Default, Deserialize)]
pub struct AgentSkillsConfig {
    #[serde(default)]
    pub enabled: Vec<String>,
}

impl PorterConfig {
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: PorterConfig = toml::from_str(&content)?;
        Ok(config)
    }
}
