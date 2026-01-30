use crate::config::McpServerConfig;
use crate::db::Database;
use crate::models::{AgentSession, AgentStatus};
use anyhow::Result;
use std::collections::HashMap;
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::{broadcast, oneshot};

/// Max time to wait for the first output line (covers MCP server startup).
const STARTUP_TIMEOUT: Duration = Duration::from_secs(30);

/// Max total time for a Claude subprocess after startup.
const SESSION_TIMEOUT: Duration = Duration::from_secs(5 * 60);

/// Events emitted by an agent session.
#[derive(Debug, Clone)]
pub enum AgentEvent {
    Output {
        session_id: String,
        content: String,
        content_type: String,
    },
    StatusChanged {
        session_id: String,
        status: AgentStatus,
    },
}

/// Options for starting a new agent session.
#[derive(Debug, Clone, Default)]
pub struct SessionOptions {
    pub working_directory: Option<String>,
    pub dangerously_skip_permissions: bool,
}

/// Manages Claude agent subprocess sessions.
pub struct AgentManager {
    db: Database,
    claude_binary: String,
    max_concurrent: usize,
    default_model: String,
    mcp_servers: HashMap<String, McpServerConfig>,
    event_tx: broadcast::Sender<AgentEvent>,
    cancel_senders: Arc<Mutex<HashMap<String, oneshot::Sender<()>>>>,
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
            cancel_senders: Arc::new(Mutex::new(HashMap::new())),
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
    pub async fn start_session(
        &self,
        prompt: &str,
        opts: SessionOptions,
    ) -> Result<AgentSession> {
        let running = self.db.list_agent_sessions(Some("running")).await?;
        if running.len() >= self.max_concurrent {
            anyhow::bail!(
                "Maximum concurrent sessions ({}) reached",
                self.max_concurrent
            );
        }

        let session = self
            .db
            .create_agent_session(
                prompt,
                &self.default_model,
                opts.working_directory.as_deref(),
                opts.dangerously_skip_permissions,
            )
            .await?;

        let session_id = session.id.clone();
        let prompt = prompt.to_string();
        let claude_binary = self.claude_binary.clone();
        let mcp_servers = self.mcp_servers.clone();
        let db = self.db.clone();
        let event_tx = self.event_tx.clone();
        let working_directory = session.working_directory.clone();
        let skip_permissions = session.dangerously_skip_permissions;

        let (cancel_tx, cancel_rx) = oneshot::channel::<()>();
        self.cancel_senders
            .lock()
            .unwrap()
            .insert(session_id.clone(), cancel_tx);
        let cancel_senders = self.cancel_senders.clone();

        tokio::spawn(async move {
            let result = run_claude_session(
                &claude_binary,
                &prompt,
                &session_id,
                &mcp_servers,
                working_directory.as_deref(),
                skip_permissions,
                &db,
                &event_tx,
                cancel_rx,
            )
            .await;

            cancel_senders.lock().unwrap().remove(&session_id);

            let final_status = match result {
                Ok(()) => AgentStatus::Completed,
                Err(e) => {
                    tracing::error!(session_id = %session_id, error = %e, "Agent session failed");
                    let _ = db
                        .add_agent_message(&session_id, "error", &e.to_string())
                        .await;
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

    /// Send a follow-up message to an existing session.
    pub async fn send_message(&self, session_id: &str, content: &str) -> Result<()> {
        let session = self
            .db
            .get_agent_session(session_id)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Session not found"))?;

        let claude_session_id = session
            .claude_session_id
            .ok_or_else(|| anyhow::anyhow!("Session has no claude_session_id"))?;

        // Store the user message
        self.db
            .add_agent_message(session_id, "user", content)
            .await?;

        // Mark session as running again
        self.db
            .update_agent_session_status(session_id, AgentStatus::Running)
            .await?;
        let _ = self.event_tx.send(AgentEvent::StatusChanged {
            session_id: session_id.to_string(),
            status: AgentStatus::Running,
        });

        let porter_session_id = session_id.to_string();
        let content = content.to_string();
        let claude_binary = self.claude_binary.clone();
        let mcp_servers = self.mcp_servers.clone();
        let db = self.db.clone();
        let event_tx = self.event_tx.clone();
        let working_directory = session.working_directory.clone();
        let skip_permissions = session.dangerously_skip_permissions;

        let (cancel_tx, cancel_rx) = oneshot::channel::<()>();
        self.cancel_senders
            .lock()
            .unwrap()
            .insert(porter_session_id.clone(), cancel_tx);
        let cancel_senders = self.cancel_senders.clone();

        tokio::spawn(async move {
            let result = resume_claude_session(
                &claude_binary,
                &claude_session_id,
                &content,
                &porter_session_id,
                &mcp_servers,
                working_directory.as_deref(),
                skip_permissions,
                &db,
                &event_tx,
                cancel_rx,
            )
            .await;

            cancel_senders.lock().unwrap().remove(&porter_session_id);

            let final_status = match result {
                Ok(()) => AgentStatus::Completed,
                Err(e) => {
                    tracing::error!(session_id = %porter_session_id, error = %e, "Agent resume failed");
                    let _ = db
                        .add_agent_message(&porter_session_id, "error", &e.to_string())
                        .await;
                    AgentStatus::Failed
                }
            };

            let _ = db
                .update_agent_session_status(&porter_session_id, final_status)
                .await;
            let _ = event_tx.send(AgentEvent::StatusChanged {
                session_id: porter_session_id,
                status: final_status,
            });
        });

        Ok(())
    }

    /// List all sessions, optionally filtered by status.
    pub async fn list_sessions(&self, status: Option<&str>) -> Result<Vec<AgentSession>> {
        self.db.list_agent_sessions(status).await
    }

    /// Get a specific session by ID.
    pub async fn get_session(&self, id: &str) -> Result<Option<AgentSession>> {
        self.db.get_agent_session(id).await
    }

    /// Cancel a running session by killing its subprocess.
    pub async fn cancel_session(&self, id: &str) -> Result<bool> {
        let cancel_tx = self.cancel_senders.lock().unwrap().remove(id);
        if let Some(tx) = cancel_tx {
            let _ = tx.send(());
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Delete a session and its messages.
    pub async fn delete_session(&self, id: &str) -> Result<bool> {
        self.db.delete_agent_session(id).await
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

    // Write and flush in a block so the BufWriter is dropped before we move `file`
    {
        use std::io::Write;
        let mut f = std::io::BufWriter::new(&file);
        serde_json::to_writer(&mut f, &mcp_json)?;
        f.flush()?;
    }

    tracing::debug!(
        path = %file.path().display(),
        servers = ?mcp_servers.keys().collect::<Vec<_>>(),
        "Wrote MCP config"
    );

    Ok(Some(file))
}

/// Resolve the working directory for a Claude subprocess.
/// If an explicit directory was provided, use it. Otherwise use a fresh temp
/// directory so Claude doesn't inherit the porter project context.
fn resolve_working_dir(
    explicit: Option<&str>,
    temp_holder: &mut Option<tempfile::TempDir>,
) -> Result<std::path::PathBuf> {
    if let Some(dir) = explicit {
        let path = std::path::PathBuf::from(dir);
        if path.is_dir() {
            return Ok(path);
        }
        anyhow::bail!("Working directory does not exist: {dir}");
    }

    // Create a temp directory so Claude starts in a blank state
    let tmp = tempfile::Builder::new()
        .prefix("porter-agent-")
        .tempdir()?;
    let path = tmp.path().to_path_buf();
    *temp_holder = Some(tmp);
    Ok(path)
}

/// Apply common flags to a Claude command: CWD, --dangerously-skip-permissions,
/// MCP config, output format, and stdio piping.
fn configure_cmd(
    cmd: &mut Command,
    cwd: &std::path::Path,
    skip_permissions: bool,
    mcp_config_file: &Option<tempfile::NamedTempFile>,
) {
    cmd.current_dir(cwd)
        .arg("--print")
        .arg("--output-format")
        .arg("stream-json")
        .arg("--verbose")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    if skip_permissions {
        cmd.arg("--dangerously-skip-permissions");
    }

    if let Some(ref config_file) = mcp_config_file {
        cmd.arg("--mcp-config").arg(config_file.path());
    }
}

/// Process streaming JSON output from a Claude subprocess line by line.
/// Extracts assistant text content, broadcasts chunks, and returns the
/// accumulated assistant text and (optionally) the Claude session ID.
async fn process_stream(
    child: &mut tokio::process::Child,
    session_id: &str,
    event_tx: &broadcast::Sender<AgentEvent>,
) -> Result<(String, Option<String>)> {
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| anyhow::anyhow!("Failed to capture stdout"))?;

    let mut reader = BufReader::new(stdout).lines();
    let mut accumulated_text = String::new();
    let mut claude_session_id: Option<String> = None;
    let mut first_event = true;

    loop {
        // Apply a short startup timeout for the first event (covers MCP server init).
        // After that, rely on the outer SESSION_TIMEOUT for the full run.
        let line = if first_event {
            match tokio::time::timeout(STARTUP_TIMEOUT, reader.next_line()).await {
                Ok(result) => result?,
                Err(_) => {
                    anyhow::bail!(
                        "No output within {} seconds — MCP server may have failed to start",
                        STARTUP_TIMEOUT.as_secs()
                    );
                }
            }
        } else {
            reader.next_line().await?
        };

        let Some(line) = line else { break };
        let parsed: serde_json::Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => continue,
        };

        first_event = false;
        let event_type = parsed["type"].as_str().unwrap_or("");

        match event_type {
            "system" => {
                // Extract session_id from init event
                if parsed["subtype"].as_str() == Some("init") {
                    if let Some(sid) = parsed["session_id"].as_str() {
                        claude_session_id = Some(sid.to_string());
                    }
                    if let Some(servers) = parsed["mcp_servers"].as_array() {
                        let names: Vec<&str> =
                            servers.iter().filter_map(|s| s.as_str()).collect();
                        tracing::info!(
                            session_id = %session_id,
                            mcp_servers = ?names,
                            "Claude session initialized"
                        );
                    }
                }
            }
            "assistant" => {
                // Parse all content blocks from the assistant message
                if let Some(content) = parsed["message"]["content"].as_array() {
                    for block in content {
                        match block["type"].as_str() {
                            Some("text") => {
                                if let Some(text) = block["text"].as_str() {
                                    accumulated_text.push_str(text);
                                    let _ = event_tx.send(AgentEvent::Output {
                                        session_id: session_id.to_string(),
                                        content: text.to_string(),
                                        content_type: "text".to_string(),
                                    });
                                }
                            }
                            Some("thinking") => {
                                if let Some(thinking) = block["thinking"].as_str() {
                                    let _ = event_tx.send(AgentEvent::Output {
                                        session_id: session_id.to_string(),
                                        content: thinking.to_string(),
                                        content_type: "thinking".to_string(),
                                    });
                                }
                            }
                            Some("tool_use") => {
                                if let Some(name) = block["name"].as_str() {
                                    let _ = event_tx.send(AgentEvent::Output {
                                        session_id: session_id.to_string(),
                                        content: name.to_string(),
                                        content_type: "tool_use".to_string(),
                                    });
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
            "result" => {
                // Check for error results (e.g. failed resume)
                if parsed["is_error"].as_bool() == Some(true) {
                    let errors = parsed["errors"]
                        .as_array()
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|e| e.as_str())
                                .collect::<Vec<_>>()
                                .join("; ")
                        })
                        .unwrap_or_else(|| "Unknown error".to_string());
                    tracing::error!(
                        session_id = %session_id,
                        "Claude error result: {errors}"
                    );
                    anyhow::bail!("{errors}");
                }
                // Final result — use its text if we haven't accumulated any
                if accumulated_text.is_empty() {
                    if let Some(text) = parsed["result"].as_str() {
                        accumulated_text = text.to_string();
                        let _ = event_tx.send(AgentEvent::Output {
                            session_id: session_id.to_string(),
                            content: text.to_string(),
                            content_type: "text".to_string(),
                        });
                    }
                }
            }
            _ => {}
        }
    }

    Ok((accumulated_text, claude_session_id))
}

/// Drain stderr and log it.
async fn drain_stderr(child: &mut tokio::process::Child, session_id: &str) {
    if let Some(mut stderr) = child.stderr.take() {
        let mut buf = String::new();
        if tokio::io::AsyncReadExt::read_to_string(&mut stderr, &mut buf)
            .await
            .is_ok()
            && !buf.is_empty()
        {
            tracing::warn!(session_id = %session_id, stderr = %buf, "Claude stderr");
        }
    }
}

/// Run a Claude subprocess with a timeout and cancellation support.
async fn run_with_timeout(
    child: &mut tokio::process::Child,
    session_id: &str,
    event_tx: &broadcast::Sender<AgentEvent>,
    cancel_rx: oneshot::Receiver<()>,
) -> Result<(String, Option<String>)> {
    tokio::select! {
        result = tokio::time::timeout(SESSION_TIMEOUT, process_stream(child, session_id, event_tx)) => {
            match result {
                Ok(inner) => inner,
                Err(_) => {
                    tracing::error!(session_id = %session_id, "Claude session timed out, killing process");
                    let _ = child.kill().await;
                    anyhow::bail!("Session timed out after {} seconds", SESSION_TIMEOUT.as_secs());
                }
            }
        }
        _ = cancel_rx => {
            tracing::info!(session_id = %session_id, "Session cancelled, killing process");
            let _ = child.kill().await;
            anyhow::bail!("Session was cancelled");
        }
    }
}

#[allow(clippy::too_many_arguments)]
async fn run_claude_session(
    claude_binary: &str,
    prompt: &str,
    session_id: &str,
    mcp_servers: &HashMap<String, McpServerConfig>,
    working_directory: Option<&str>,
    skip_permissions: bool,
    db: &Database,
    event_tx: &broadcast::Sender<AgentEvent>,
    cancel_rx: oneshot::Receiver<()>,
) -> Result<()> {
    db.add_agent_message(session_id, "user", prompt).await?;

    let mcp_config_file = build_mcp_config(mcp_servers)?;
    let mut _temp_dir = None;
    let cwd = resolve_working_dir(working_directory, &mut _temp_dir)?;

    let mut cmd = Command::new(claude_binary);
    configure_cmd(&mut cmd, &cwd, skip_permissions, &mcp_config_file);

    // Append the prompt as a system prompt if MCP servers are configured,
    // so the agent knows what tools are available via MCP.
    if !mcp_servers.is_empty() {
        let server_list: Vec<&str> = mcp_servers.keys().map(|s| s.as_str()).collect();
        let system_note = format!(
            "You have access to MCP servers: {}. Use them when relevant.",
            server_list.join(", ")
        );
        cmd.arg("--append-system-prompt").arg(&system_note);
    }

    cmd.arg(prompt);

    tracing::info!(
        session_id = %session_id,
        cwd = %cwd.display(),
        mcp = ?mcp_servers.keys().collect::<Vec<_>>(),
        skip_permissions,
        "Starting Claude session"
    );

    let mut child = cmd.spawn()?;
    let (text, claude_sid) = run_with_timeout(&mut child, session_id, event_tx, cancel_rx).await?;

    if let Some(ref csid) = claude_sid {
        db.set_claude_session_id(session_id, csid).await?;
    }

    drain_stderr(&mut child, session_id).await;
    let status = child.wait().await?;

    if !text.is_empty() {
        db.add_agent_message(session_id, "assistant", &text).await?;
    }

    if !status.success() {
        anyhow::bail!("Claude process exited with status: {}", status);
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn resume_claude_session(
    claude_binary: &str,
    claude_session_id: &str,
    prompt: &str,
    session_id: &str,
    mcp_servers: &HashMap<String, McpServerConfig>,
    working_directory: Option<&str>,
    skip_permissions: bool,
    db: &Database,
    event_tx: &broadcast::Sender<AgentEvent>,
    cancel_rx: oneshot::Receiver<()>,
) -> Result<()> {
    let mcp_config_file = build_mcp_config(mcp_servers)?;
    let mut _temp_dir = None;
    let cwd = resolve_working_dir(working_directory, &mut _temp_dir)?;

    let mut cmd = Command::new(claude_binary);
    cmd.arg("--resume").arg(claude_session_id);
    configure_cmd(&mut cmd, &cwd, skip_permissions, &mcp_config_file);
    cmd.arg(prompt);

    tracing::info!(
        session_id = %session_id,
        claude_session_id = %claude_session_id,
        cwd = %cwd.display(),
        mcp = ?mcp_servers.keys().collect::<Vec<_>>(),
        skip_permissions,
        "Resuming Claude session"
    );

    let mut child = cmd.spawn()?;
    let (text, _) = run_with_timeout(&mut child, session_id, event_tx, cancel_rx).await?;

    drain_stderr(&mut child, session_id).await;
    let status = child.wait().await?;

    if !text.is_empty() {
        db.add_agent_message(session_id, "assistant", &text).await?;
    }

    if !status.success() {
        anyhow::bail!("Claude resume exited with status: {}", status);
    }

    Ok(())
}
