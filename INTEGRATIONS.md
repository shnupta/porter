# Porter Integration System

Porter uses a hybrid integration architecture:

- **Built-in integrations** (Rust) — for core functionality that needs server-side background processing, direct database access, or custom REST API endpoints.
- **MCP servers** (config-only) — for giving Claude agent sessions access to external APIs and tools. No Rust code needed.

## When to use which

| Use case | Approach |
|---|---|
| Claude needs to call an external API (Slack, OpenTable, etc.) | MCP server |
| You need background polling (check email every 5 min) | Built-in integration |
| You need custom REST endpoints in the Porter API | Built-in integration |
| You found an existing MCP server package on npm | MCP server |
| You need to store state in Porter's database | Built-in integration |

Most new integrations should be **MCP servers**. Only write a built-in integration when you need server-side behavior that Claude can't drive on its own.

---

## Adding an MCP server integration

This is the common case. No code changes — just config.

### 1. Add the config to your TOML file

```toml
# config/home.toml

[agents.mcp.slack]
command = "npx"
args = ["-y", "@anthropic/slack-mcp"]
env = { SLACK_TOKEN = "env:PORTER_SLACK_TOKEN" }
```

- **`command`** — the executable to run (e.g. `npx`, `node`, `python`)
- **`args`** — arguments passed to the command
- **`env`** — environment variables for the MCP server process. Values prefixed with `env:` are resolved from the host environment at runtime, so secrets stay out of config files.

### 2. Set any required environment variables

```bash
export PORTER_SLACK_TOKEN="xoxb-your-token-here"
```

### 3. Start Porter

```bash
porter serve --config config/home.toml
```

That's it. When a Claude agent session starts, Porter:

1. Reads all `[agents.mcp.*]` entries from the config
2. Builds a temporary JSON file in the Claude MCP config format:
   ```json
   {
     "mcpServers": {
       "slack": {
         "command": "npx",
         "args": ["-y", "@anthropic/slack-mcp"],
         "env": { "SLACK_TOKEN": "xoxb-your-token-here" }
       }
     }
   }
   ```
3. Passes it to the Claude CLI via `--mcp-config /tmp/porter-mcp-XXXXX.json`
4. Claude can now use the MCP server's tools during the session

### Finding MCP servers

Any MCP server that works with the Claude CLI will work here. Look for packages on npm (`npx -y <package>`), or point to a local script.

---

## Adding a built-in integration

Built-in integrations implement the `Integration` trait in Rust. Use this when you need behavior that runs on the server independent of Claude sessions.

### 1. Create the module

```
crates/porter-integrations/src/myfeature/mod.rs
```

```rust
use anyhow::Result;
use async_trait::async_trait;
use porter_core::integrations::{
    Action, ActionResult, Capability, Integration, IntegrationConfig,
};
use porter_core::models::Notification;

pub struct MyFeatureIntegration {
    // any state your integration needs
}

impl MyFeatureIntegration {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Integration for MyFeatureIntegration {
    fn id(&self) -> &str {
        "myfeature"
    }

    fn name(&self) -> &str {
        "My Feature"
    }

    async fn init(&mut self, _config: &IntegrationConfig) -> Result<()> {
        // Setup: open connections, load state, etc.
        Ok(())
    }

    async fn handle(&self, action: Action) -> Result<ActionResult> {
        // Handle actions triggered by other parts of the system.
        match action.name.as_str() {
            "do_something" => {
                // ...
                Ok(ActionResult {
                    success: true,
                    message: "Done".into(),
                    data: None,
                })
            }
            _ => Ok(ActionResult {
                success: false,
                message: format!("Unknown action: {}", action.name),
                data: None,
            }),
        }
    }

    async fn tick(&self) -> Result<Vec<Notification>> {
        // Called periodically for background work (polling APIs, etc.)
        // Return any notifications that should be surfaced to the user.
        Ok(vec![])
    }

    fn capabilities(&self) -> Vec<Capability> {
        vec![Capability {
            name: "do_something".into(),
            description: "Does something useful".into(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "input": { "type": "string" }
                }
            }),
        }]
    }
}
```

### 2. Export the module

```rust
// crates/porter-integrations/src/lib.rs
mod myfeature;
```

### 3. Register it

```rust
// crates/porter-integrations/src/lib.rs
pub fn register_builtin_integrations(registry: &mut IntegrationRegistry, enabled: &[String]) {
    if enabled.contains(&"tasks".to_string()) {
        registry.register(Arc::new(tasks::TaskIntegration::new()));
    }
    if enabled.contains(&"myfeature".to_string()) {
        registry.register(Arc::new(myfeature::MyFeatureIntegration::new()));
    }
}
```

### 4. Enable in config

```toml
[integrations]
enabled = ["tasks", "myfeature"]
```

### 5. (Optional) Add API endpoints

If your integration needs REST endpoints, add a route module in `porter-server/src/api/` and merge it into the router in `api/mod.rs`. The integration is accessible via `AppState.integration_registry`.

---

## Current architecture

```
                    ┌─────────────────────────────────────┐
                    │           Porter Server              │
                    │                                      │
  REST API ◄────── │  IntegrationRegistry                  │
  /api/tasks       │    ├── TaskIntegration (built-in)     │
  /api/integrations│    └── ... (future built-in)          │
                    │                                      │
  Claude CLI ◄──── │  AgentManager                         │
  (subprocess)     │    ├── --mcp-config temp.json         │
                    │    │     ├── slack (MCP server)       │
                    │    │     ├── flights (MCP server)     │
                    │    │     └── ... (from TOML config)   │
                    │    │                                  │
                    │    └── sessions tracked in SQLite     │
                    │                                      │
  WebSocket ◄───── │  Broadcast (task/agent events)        │
                    └─────────────────────────────────────┘
```

Built-in integrations live inside the server process. MCP servers are spawned by the Claude CLI as separate child processes — Porter just tells Claude where to find them.

---

## Known gaps / TODO

These are things that exist in the trait or config but aren't fully wired up yet:

1. **`tick()` isn't called** — there's no background loop invoking `tick()` on registered integrations. Needs a tokio task that runs on an interval and calls each integration's `tick()`, then broadcasts any returned notifications.

2. **Per-integration config** — `IntegrationConfig` exists but is never populated from TOML. To support config like `[integrations.myfeature] api_key = "..."`, the config parser needs to extract per-integration sections and pass them through `init()`.

3. **No bridge from integrations to Claude** — built-in integrations declare `capabilities()` but Claude can't invoke them. If you want Claude to use a built-in integration, you'd either need to wrap it as an MCP server or add a generic MCP server in the agent manager that proxies to the integration registry.

4. **`handle()` has no caller** — currently nothing in the server calls `integration.handle()`. It's defined in the trait but unused. The intent is for it to be called by API routes or by a future integration-to-Claude bridge.

5. **No lifecycle cleanup** — the trait has no `shutdown()` method. Integrations holding connections or temp files have no cleanup hook.

6. **Silent env var failures** — if an MCP server config references `env:MISSING_VAR`, it silently resolves to an empty string. Should probably warn or fail at startup.
