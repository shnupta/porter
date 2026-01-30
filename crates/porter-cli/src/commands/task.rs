use colored::Colorize;
use porter_core::models::{CreateTask, Task, TaskPriority};

pub async fn create(server: &str, title: &str, priority: &str) -> anyhow::Result<()> {
    let client = reqwest::Client::new();

    let priority = TaskPriority::from_str(priority);
    let input = CreateTask {
        title: title.to_string(),
        description: None,
        priority,
        tags: None,
        due_date: None,
    };

    let resp = client
        .post(format!("{server}/api/tasks"))
        .json(&input)
        .send()
        .await?;

    if resp.status().is_success() {
        let task: Task = resp.json().await?;
        println!("{} Created task: {}", "✓".green(), task.title);
        println!("  ID: {}", task.id.dimmed());
    } else {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        anyhow::bail!("Failed to create task: {} {}", status, body);
    }

    Ok(())
}

pub async fn list(server: &str, status: Option<&str>) -> anyhow::Result<()> {
    let client = reqwest::Client::new();

    let mut url = format!("{server}/api/tasks");
    if let Some(s) = status {
        url = format!("{url}?status={s}");
    }

    let resp = client.get(&url).send().await?;

    if resp.status().is_success() {
        let tasks: Vec<Task> = resp.json().await?;

        if tasks.is_empty() {
            println!("{}", "No tasks found.".dimmed());
            return Ok(());
        }

        println!("{}", "Tasks:".bold());
        for task in &tasks {
            let status_icon = match task.status {
                porter_core::models::TaskStatus::Pending => "○".yellow(),
                porter_core::models::TaskStatus::InProgress => "◉".blue(),
                porter_core::models::TaskStatus::Completed => "●".green(),
                porter_core::models::TaskStatus::Cancelled => "✕".red(),
            };
            let priority_str = match task.priority {
                porter_core::models::TaskPriority::Urgent => "!!!".red(),
                porter_core::models::TaskPriority::High => "!!".red(),
                porter_core::models::TaskPriority::Medium => "!".yellow(),
                porter_core::models::TaskPriority::Low => " ".normal(),
            };
            println!(
                "  {status_icon} {priority_str} {} {}",
                task.title,
                task.id[..8].dimmed()
            );
        }
        println!("\n  {} tasks total", tasks.len().to_string().bold());
    } else {
        anyhow::bail!("Failed to list tasks: {}", resp.status());
    }

    Ok(())
}
