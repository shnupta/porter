use colored::Colorize;
use porter_core::models::AgentSession;
use serde_json::json;

pub async fn start(server: &str, prompt: &str) -> anyhow::Result<()> {
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{server}/api/agents"))
        .json(&json!({ "prompt": prompt }))
        .send()
        .await?;

    if resp.status().is_success() {
        let session: AgentSession = resp.json().await?;
        println!("{} Started agent session", "✓".green());
        println!("  ID: {}", session.id.dimmed());
        println!("  Model: {}", session.model);
        println!("  Prompt: {}", session.prompt);
    } else {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        anyhow::bail!("Failed to start agent session: {} {}", status, body);
    }

    Ok(())
}

pub async fn list(server: &str, status: Option<&str>) -> anyhow::Result<()> {
    let client = reqwest::Client::new();

    let mut url = format!("{server}/api/agents");
    if let Some(s) = status {
        url = format!("{url}?status={s}");
    }

    let resp = client.get(&url).send().await?;

    if resp.status().is_success() {
        let sessions: Vec<AgentSession> = resp.json().await?;

        if sessions.is_empty() {
            println!("{}", "No agent sessions found.".dimmed());
            return Ok(());
        }

        println!("{}", "Agent Sessions:".bold());
        for session in &sessions {
            let status_icon = match session.status {
                porter_core::models::AgentStatus::Running => "▶".green(),
                porter_core::models::AgentStatus::Paused => "⏸".yellow(),
                porter_core::models::AgentStatus::Completed => "✓".green(),
                porter_core::models::AgentStatus::Failed => "✕".red(),
            };
            let prompt_preview = if session.prompt.len() > 60 {
                format!("{}...", &session.prompt[..57])
            } else {
                session.prompt.clone()
            };
            println!(
                "  {status_icon} {} {} ({})",
                session.id[..8].dimmed(),
                prompt_preview,
                session.model.dimmed()
            );
        }
    } else {
        anyhow::bail!("Failed to list sessions: {}", resp.status());
    }

    Ok(())
}
