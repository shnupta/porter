use sqlx::SqlitePool;

pub async fn run_migrations(pool: &SqlitePool) -> anyhow::Result<()> {
    sqlx::raw_sql(
        "
        CREATE TABLE IF NOT EXISTS tasks (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            description TEXT,
            status TEXT NOT NULL DEFAULT 'pending',
            priority TEXT NOT NULL DEFAULT 'medium',
            tags TEXT NOT NULL DEFAULT '[]',
            due_date TEXT,
            skill_id TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS agent_sessions (
            id TEXT PRIMARY KEY,
            prompt TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'running',
            model TEXT NOT NULL DEFAULT 'opus',
            started_at TEXT NOT NULL,
            completed_at TEXT
        );

        CREATE TABLE IF NOT EXISTS agent_messages (
            id TEXT PRIMARY KEY,
            session_id TEXT NOT NULL,
            role TEXT NOT NULL,
            content TEXT NOT NULL,
            timestamp TEXT NOT NULL,
            FOREIGN KEY (session_id) REFERENCES agent_sessions(id)
        );

        CREATE TABLE IF NOT EXISTS notifications (
            id TEXT PRIMARY KEY,
            notification_type TEXT NOT NULL,
            message TEXT NOT NULL,
            read INTEGER NOT NULL DEFAULT 0,
            skill_id TEXT,
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS documents (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            doc_type TEXT NOT NULL,
            received_date TEXT,
            notes TEXT,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS skills_state (
            skill_id TEXT NOT NULL,
            key TEXT NOT NULL,
            value TEXT NOT NULL,
            updated_at TEXT NOT NULL,
            PRIMARY KEY (skill_id, key)
        );

        CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks(status);
        CREATE INDEX IF NOT EXISTS idx_tasks_priority ON tasks(priority);
        CREATE INDEX IF NOT EXISTS idx_agent_sessions_status ON agent_sessions(status);
        CREATE INDEX IF NOT EXISTS idx_agent_messages_session ON agent_messages(session_id);
        CREATE INDEX IF NOT EXISTS idx_notifications_read ON notifications(read);
        ",
    )
    .execute(pool)
    .await?;

    tracing::info!("Database migrations completed");
    Ok(())
}
