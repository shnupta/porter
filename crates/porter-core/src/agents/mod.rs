use crate::config::McpServerConfig;
use crate::db::Database;
use crate::models::{AgentSession, AgentStatus};
use anyhow::Result;
use std::collections::HashMap;
use tokio::process::Command;
use tokio::sync::broadcast;

/// Events emitted by an agent session.
#[derive(Debug, Clone)]
pub enum AgentEvent {
    Output { session_id: String, content: String },
    StatusChanged { session_id: String, status: AgentStatus },
}

/// Manages Claude agent subprocess sessions.
pub struct AgentManager {
    db: Database,
    claude_binary: String,
    max_concurrent: usize,
    default_model: String,
    mcp_servers: HashMap<String, McpServerConfig>,
    event_tx: broadcast::Sender<AgentEvent>,
}

impl AgentManager {
    pub fn new(
        db: Database,
        claude_binary: String,
        max_concurrent: usize,
        default_model: String,
        mcp_servers: HashMap<String, McpServerConfig>,
    ) -> Self {
        let (event_tx, _) = broadcast::channel(256);
        Self {
            db,
            claude_binary,
            max_concurrent,
            default_model,
            mcp_servers,
            event_tx,
        }
    }

    /// Subscribe to agent events (for WebSocket broadcasting).
    pub fn subscribe(&self) -> broadcast::Receiver<AgentEvent> {
        self.event_tx.subscribe()
    }

    /// List configured MCP server names.
    pub fn mcp_server_names(&self) -> Vec<String> {
        self.mcp_servers.keys().cloned().collect()
    }

    /// Start a new Claude agent session.
    pub async fn start_session(&self, prompt: &str) -> Result<AgentSession> {
        let running = self.db.list_agent_sessions(Some("running")).await?;
        if running.len() >= self.max_concurrent {
            anyhow::bail!(
                "Maximum concurrent sessions ({}) reached",
                self.max_concurrent
            );
        }

        let session = self
            .db
            .create_agent_session(prompt, &self.default_model)
            .await?;

        let session_id = session.id.clone();
        let prompt = prompt.to_string();
        let claude_binary = self.claude_binary.clone();
        let mcp_servers = self.mcp_servers.clone();
        let db = self.db.clone();
        let event_tx = self.event_tx.clone();

        tokio::spawn(async move {
            let result = run_claude_session(
                &claude_binary,
                &prompt,
                &session_id,
                &mcp_servers,
                &db,
                &event_tx,
            )
            .await;

            let final_status = match result {
                Ok(()) => AgentStatus::Completed,
                Err(e) => {
                    tracing::error!(session_id = %session_id, error = %e, "Agent session failed");
                    AgentStatus::Failed
                }
            };

            let _ = db
                .update_agent_session_status(&session_id, final_status)
                .await;
            let _ = event_tx.send(AgentEvent::StatusChanged {
                session_id,
                status: final_status,
            });
        });

        Ok(session)
    }

    /// List all sessions, optionally filtered by status.
    pub async fn list_sessions(&self, status: Option<&str>) -> Result<Vec<AgentSession>> {
        self.db.list_agent_sessions(status).await
    }

    /// Get a specific session by ID.
    pub async fn get_session(&self, id: &str) -> Result<Option<AgentSession>> {
        self.db.get_agent_session(id).await
    }
}

/// Build a temporary MCP config JSON file for the Claude CLI.
fn build_mcp_config(
    mcp_servers: &HashMap<String, McpServerConfig>,
) -> Result<Option<tempfile::NamedTempFile>> {
    if mcp_servers.is_empty() {
        return Ok(None);
    }

    let mut servers = serde_json::Map::new();

    for (name, config) in mcp_servers {
        let mut env_map = serde_json::Map::new();
        for (key, value) in &config.env {
            let resolved = if let Some(env_key) = value.strip_prefix("env:") {
                std::env::var(env_key).unwrap_or_default()
            } else {
                value.clone()
            };
            env_map.insert(key.clone(), serde_json::Value::String(resolved));
        }

        let server = serde_json::json!({
            "command": config.command,
            "args": config.args,
            "env": env_map,
        });

        servers.insert(name.clone(), server);
    }

    let mcp_json = serde_json::json!({ "mcpServers": servers });

    let file = tempfile::Builder::new()
        .prefix("porter-mcp-")
        .suffix(".json")
        .tempfile()?;

    serde_json::to_writer(&file, &mcp_json)?;

    Ok(Some(file))
}

async fn run_claude_session(
    claude_binary: &str,
    prompt: &str,
    session_id: &str,
    mcp_servers: &HashMap<String, McpServerConfig>,
    db: &Database,
    event_tx: &broadcast::Sender<AgentEvent>,
) -> Result<()> {
    db.add_agent_message(session_id, "user", prompt).await?;

    let mcp_config_file = build_mcp_config(mcp_servers)?;

    let mut cmd = Command::new(claude_binary);
    cmd.arg("--print").arg(prompt);

    if let Some(ref config_file) = mcp_config_file {
        cmd.arg("--mcp-config").arg(config_file.path());
    }

    let output = cmd.output().await?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !stdout.is_empty() {
        db.add_agent_message(session_id, "assistant", &stdout)
            .await?;
        let _ = event_tx.send(AgentEvent::Output {
            session_id: session_id.to_string(),
            content: stdout,
        });
    }

    if !stderr.is_empty() {
        tracing::warn!(session_id = %session_id, stderr = %stderr, "Claude stderr output");
    }

    if !output.status.success() {
        anyhow::bail!("Claude process exited with status: {}", output.status);
    }

    Ok(())
}
