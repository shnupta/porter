use colored::Colorize;
use porter_core::models::ServerStatus;

pub async fn run(server: &str) -> anyhow::Result<()> {
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("{server}/api/status"))
        .send()
        .await;

    match resp {
        Ok(resp) if resp.status().is_success() => {
            let status: ServerStatus = resp.json().await?;
            println!("{}", "Porter Status".bold());
            println!("  Instance: {}", status.instance_name.cyan());
            println!("  Version:  {}", status.version);
            println!("  Uptime:   {}s", status.uptime_seconds);
            println!(
                "  Skills:   {}",
                if status.active_skills.is_empty() {
                    "none".dimmed().to_string()
                } else {
                    status.active_skills.join(", ")
                }
            );
            println!("  Sessions: {}", status.active_agent_sessions);
            println!("  Pending:  {} tasks", status.pending_tasks);
        }
        Ok(resp) => {
            anyhow::bail!("Server returned error: {}", resp.status());
        }
        Err(e) => {
            println!("{} Cannot connect to Porter server at {server}", "✕".red());
            println!("  {}", e.to_string().dimmed());
            println!(
                "\n  Start the server with: {}",
                "porter serve --config config/home.toml".cyan()
            );
        }
    }

    Ok(())
}
