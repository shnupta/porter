use crate::models::*;
use chrono::Utc;
use sqlx::sqlite::SqliteRow;
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    // ── Tasks ──

    pub async fn create_task(&self, input: CreateTask) -> anyhow::Result<Task> {
        let task = Task::new(input);
        let tags_json = serde_json::to_string(&task.tags)?;

        sqlx::query(
            "INSERT INTO tasks (id, title, description, status, priority, tags, due_date, integration_id, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(&task.id)
        .bind(&task.title)
        .bind(&task.description)
        .bind(task.status.as_str())
        .bind(task.priority.as_str())
        .bind(&tags_json)
        .bind(task.due_date.map(|d| d.to_rfc3339()))
        .bind(&task.integration_id)
        .bind(task.created_at.to_rfc3339())
        .bind(task.updated_at.to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(task)
    }

    pub async fn get_task(&self, id: &str) -> anyhow::Result<Option<Task>> {
        let row = sqlx::query("SELECT * FROM tasks WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        match row {
            Some(row) => Ok(Some(task_from_row(&row)?)),
            None => Ok(None),
        }
    }

    pub async fn list_tasks(&self, status: Option<&str>) -> anyhow::Result<Vec<Task>> {
        let rows = if let Some(status) = status {
            sqlx::query("SELECT * FROM tasks WHERE status = ? ORDER BY created_at DESC")
                .bind(status)
                .fetch_all(&self.pool)
                .await?
        } else {
            sqlx::query("SELECT * FROM tasks ORDER BY created_at DESC")
                .fetch_all(&self.pool)
                .await?
        };

        rows.iter().map(task_from_row).collect()
    }

    pub async fn update_task(&self, id: &str, input: UpdateTask) -> anyhow::Result<Option<Task>> {
        let existing = self.get_task(id).await?;
        let Some(mut task) = existing else {
            return Ok(None);
        };

        if let Some(title) = input.title {
            task.title = title;
        }
        if let Some(desc) = input.description {
            task.description = Some(desc);
        }
        if let Some(status) = input.status {
            task.status = status;
        }
        if let Some(priority) = input.priority {
            task.priority = priority;
        }
        if let Some(tags) = input.tags {
            task.tags = tags;
        }
        if input.due_date.is_some() {
            task.due_date = input.due_date;
        }
        task.updated_at = Utc::now();

        let tags_json = serde_json::to_string(&task.tags)?;
        sqlx::query(
            "UPDATE tasks SET title = ?, description = ?, status = ?, priority = ?, tags = ?, due_date = ?, updated_at = ? WHERE id = ?"
        )
        .bind(&task.title)
        .bind(&task.description)
        .bind(task.status.as_str())
        .bind(task.priority.as_str())
        .bind(&tags_json)
        .bind(task.due_date.map(|d| d.to_rfc3339()))
        .bind(task.updated_at.to_rfc3339())
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(Some(task))
    }

    pub async fn delete_task(&self, id: &str) -> anyhow::Result<bool> {
        let result = sqlx::query("DELETE FROM tasks WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn count_tasks_by_status(&self, status: &str) -> anyhow::Result<i64> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM tasks WHERE status = ?")
            .bind(status)
            .fetch_one(&self.pool)
            .await?;
        Ok(row.get::<i64, _>("count"))
    }

    // ── Agent Sessions ──

    pub async fn create_agent_session(
        &self,
        prompt: &str,
        model: &str,
        working_directory: Option<&str>,
        dangerously_skip_permissions: bool,
    ) -> anyhow::Result<AgentSession> {
        let session = AgentSession {
            id: Uuid::new_v4().to_string(),
            prompt: prompt.to_string(),
            status: AgentStatus::Running,
            model: model.to_string(),
            claude_session_id: None,
            working_directory: working_directory.map(String::from),
            dangerously_skip_permissions,
            started_at: Utc::now(),
            completed_at: None,
        };

        sqlx::query(
            "INSERT INTO agent_sessions (id, prompt, status, model, working_directory, dangerously_skip_permissions, started_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&session.id)
        .bind(&session.prompt)
        .bind(session.status.as_str())
        .bind(&session.model)
        .bind(&session.working_directory)
        .bind(session.dangerously_skip_permissions)
        .bind(session.started_at.to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(session)
    }

    pub async fn get_agent_session(&self, id: &str) -> anyhow::Result<Option<AgentSession>> {
        let row = sqlx::query("SELECT * FROM agent_sessions WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        match row {
            Some(row) => Ok(Some(agent_session_from_row(&row)?)),
            None => Ok(None),
        }
    }

    pub async fn list_agent_sessions(
        &self,
        status: Option<&str>,
    ) -> anyhow::Result<Vec<AgentSession>> {
        let rows = if let Some(status) = status {
            sqlx::query(
                "SELECT * FROM agent_sessions WHERE status = ? ORDER BY started_at DESC",
            )
            .bind(status)
            .fetch_all(&self.pool)
            .await?
        } else {
            sqlx::query("SELECT * FROM agent_sessions ORDER BY started_at DESC")
                .fetch_all(&self.pool)
                .await?
        };

        rows.iter().map(agent_session_from_row).collect()
    }

    pub async fn update_agent_session_status(
        &self,
        id: &str,
        status: AgentStatus,
    ) -> anyhow::Result<bool> {
        let completed_at = if matches!(status, AgentStatus::Completed | AgentStatus::Failed) {
            Some(Utc::now().to_rfc3339())
        } else {
            None
        };

        let result = sqlx::query(
            "UPDATE agent_sessions SET status = ?, completed_at = COALESCE(?, completed_at) WHERE id = ?",
        )
        .bind(status.as_str())
        .bind(completed_at)
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn add_agent_message(
        &self,
        session_id: &str,
        role: &str,
        content: &str,
    ) -> anyhow::Result<AgentMessage> {
        let msg = AgentMessage {
            id: Uuid::new_v4().to_string(),
            session_id: session_id.to_string(),
            role: role.to_string(),
            content: content.to_string(),
            timestamp: Utc::now(),
        };

        sqlx::query(
            "INSERT INTO agent_messages (id, session_id, role, content, timestamp) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(&msg.id)
        .bind(&msg.session_id)
        .bind(&msg.role)
        .bind(&msg.content)
        .bind(msg.timestamp.to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(msg)
    }

    pub async fn delete_agent_session(&self, id: &str) -> anyhow::Result<bool> {
        sqlx::query("DELETE FROM agent_messages WHERE session_id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        let result = sqlx::query("DELETE FROM agent_sessions WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn set_claude_session_id(
        &self,
        session_id: &str,
        claude_session_id: &str,
    ) -> anyhow::Result<bool> {
        let result = sqlx::query(
            "UPDATE agent_sessions SET claude_session_id = ? WHERE id = ?",
        )
        .bind(claude_session_id)
        .bind(session_id)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn get_agent_messages(
        &self,
        session_id: &str,
    ) -> anyhow::Result<Vec<AgentMessage>> {
        let rows =
            sqlx::query("SELECT * FROM agent_messages WHERE session_id = ? ORDER BY timestamp ASC")
                .bind(session_id)
                .fetch_all(&self.pool)
                .await?;

        rows.iter().map(agent_message_from_row).collect()
    }

    // ── Notifications ──

    // ── Integration State ──

    pub async fn get_integration_state(
        &self,
        integration_id: &str,
        key: &str,
    ) -> anyhow::Result<Option<String>> {
        let row = sqlx::query(
            "SELECT value FROM integrations_state WHERE integration_id = ? AND key = ?",
        )
        .bind(integration_id)
        .bind(key)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.get("value")))
    }

    pub async fn set_integration_state(
        &self,
        integration_id: &str,
        key: &str,
        value: &str,
    ) -> anyhow::Result<()> {
        let now = Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT INTO integrations_state (integration_id, key, value, updated_at)
             VALUES (?, ?, ?, ?)
             ON CONFLICT (integration_id, key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
        )
        .bind(integration_id)
        .bind(key)
        .bind(value)
        .bind(&now)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn delete_integration_state(
        &self,
        integration_id: &str,
        key: &str,
    ) -> anyhow::Result<bool> {
        let result = sqlx::query(
            "DELETE FROM integrations_state WHERE integration_id = ? AND key = ?",
        )
        .bind(integration_id)
        .bind(key)
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected() > 0)
    }

    // ── Notifications ──

    pub async fn create_notification(
        &self,
        notification_type: &str,
        message: &str,
        integration_id: Option<&str>,
    ) -> anyhow::Result<Notification> {
        let notification = Notification {
            id: Uuid::new_v4().to_string(),
            notification_type: notification_type.to_string(),
            message: message.to_string(),
            read: false,
            integration_id: integration_id.map(String::from),
            created_at: Utc::now(),
        };

        sqlx::query(
            "INSERT INTO notifications (id, notification_type, message, read, integration_id, created_at) VALUES (?, ?, ?, 0, ?, ?)",
        )
        .bind(&notification.id)
        .bind(&notification.notification_type)
        .bind(&notification.message)
        .bind(&notification.integration_id)
        .bind(notification.created_at.to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(notification)
    }
}

// ── Row mapping helpers ──

fn task_from_row(row: &SqliteRow) -> anyhow::Result<Task> {
    let tags_str: String = row.get("tags");
    let tags: Vec<String> = serde_json::from_str(&tags_str).unwrap_or_default();

    let due_date: Option<String> = row.get("due_date");
    let due_date = due_date
        .and_then(|d| chrono::DateTime::parse_from_rfc3339(&d).ok())
        .map(|d| d.with_timezone(&Utc));

    let created_at: String = row.get("created_at");
    let updated_at: String = row.get("updated_at");
    let status_str: String = row.get("status");
    let priority_str: String = row.get("priority");

    Ok(Task {
        id: row.get("id"),
        title: row.get("title"),
        description: row.get("description"),
        status: TaskStatus::from_str(&status_str).unwrap_or(TaskStatus::Pending),
        priority: TaskPriority::from_str(&priority_str).unwrap_or(TaskPriority::Medium),
        tags,
        due_date,
        integration_id: row.get("integration_id"),
        created_at: chrono::DateTime::parse_from_rfc3339(&created_at)?
            .with_timezone(&Utc),
        updated_at: chrono::DateTime::parse_from_rfc3339(&updated_at)?
            .with_timezone(&Utc),
    })
}

fn agent_session_from_row(row: &SqliteRow) -> anyhow::Result<AgentSession> {
    let started_at: String = row.get("started_at");
    let completed_at: Option<String> = row.get("completed_at");
    let status_str: String = row.get("status");

    let skip_perms: bool = row.try_get("dangerously_skip_permissions").unwrap_or(false);

    Ok(AgentSession {
        id: row.get("id"),
        prompt: row.get("prompt"),
        status: AgentStatus::from_str(&status_str).unwrap_or(AgentStatus::Running),
        model: row.get("model"),
        claude_session_id: row.get("claude_session_id"),
        working_directory: row.get("working_directory"),
        dangerously_skip_permissions: skip_perms,
        started_at: chrono::DateTime::parse_from_rfc3339(&started_at)?
            .with_timezone(&Utc),
        completed_at: completed_at
            .and_then(|d| chrono::DateTime::parse_from_rfc3339(&d).ok())
            .map(|d| d.with_timezone(&Utc)),
    })
}

fn agent_message_from_row(row: &SqliteRow) -> anyhow::Result<AgentMessage> {
    let timestamp: String = row.get("timestamp");

    Ok(AgentMessage {
        id: row.get("id"),
        session_id: row.get("session_id"),
        role: row.get("role"),
        content: row.get("content"),
        timestamp: chrono::DateTime::parse_from_rfc3339(&timestamp)?
            .with_timezone(&Utc),
    })
}
