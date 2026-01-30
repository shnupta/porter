use async_trait::async_trait;
use porter_core::models::Notification;
use porter_core::skills::*;

pub struct TaskSkill;

impl TaskSkill {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl Skill for TaskSkill {
    fn id(&self) -> &str {
        "tasks"
    }

    fn name(&self) -> &str {
        "Task Management"
    }

    async fn init(&mut self, _config: &SkillConfig) -> anyhow::Result<()> {
        tracing::info!("Task skill initialized");
        Ok(())
    }

    async fn handle(&self, action: Action) -> anyhow::Result<ActionResult> {
        match action.name.as_str() {
            "create" | "list" | "update" | "delete" => Ok(ActionResult {
                success: true,
                message: format!("Task action '{}' handled", action.name),
                data: Some(action.params),
            }),
            _ => Ok(ActionResult {
                success: false,
                message: format!("Unknown task action: {}", action.name),
                data: None,
            }),
        }
    }

    async fn tick(&self) -> anyhow::Result<Vec<Notification>> {
        Ok(vec![])
    }

    fn capabilities(&self) -> Vec<Capability> {
        vec![
            Capability {
                name: "create_task".to_string(),
                description: "Create a new task".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "title": { "type": "string" },
                        "description": { "type": "string" },
                        "priority": { "type": "string", "enum": ["low", "medium", "high", "urgent"] }
                    },
                    "required": ["title"]
                }),
            },
            Capability {
                name: "list_tasks".to_string(),
                description: "List tasks with optional status filter".to_string(),
                parameters: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "status": { "type": "string", "enum": ["pending", "in_progress", "completed", "cancelled"] }
                    }
                }),
            },
        ]
    }
}
