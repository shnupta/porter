# Porter Development Plan

## Current State (as of 2025-01-29)

### What's Built
- **Core infrastructure**: Cargo workspace, SQLite DB, TOML config, Axum REST API, WebSocket
- **Task system**: Full CRUD API + frontend UI with real-time updates
- **Agent system**: Spawns Claude CLI subprocesses with MCP server configs, stores sessions/messages in DB
- **Integration system**: Trait-based built-in integrations with DB state, env-resolved config, background tick loops, webhook endpoint
- **Frontend**: Next.js + shadcn/ui dashboard, task management, dark mode

### Completed
- [x] Phase 1: Core infrastructure
- [x] Phase 2: Task CRUD UI + real-time WebSocket updates
- [x] Phase 3: Integration system (tick loops, webhooks, DB state, env config)
- [x] Phase 5.1: Added `@anthropic/mcp-server-fetch` to agent MCP config

### Known Gaps in Existing Code
- Agent events (AgentEvent) are broadcast internally but NOT forwarded to WebSocket clients
- Agents receive only prompt text — no task data or integration context
- `handle()` on integrations is defined but never called from anywhere
- No graceful shutdown for tick loops
- INTEGRATIONS.md is stale (still lists tick loop and config as unimplemented)

---

## Phase 4: Interactive Agent Chat + Integration Bridge

**Goal**: Make agent sessions visible and interactive in the UI, and give Claude
access to Porter's own data via MCP.

### 4.1 — Streaming Agent Sessions (Backend)

Switch from one-shot `claude --print` to streaming with `--output-format stream-json --verbose`.

**Key changes to `AgentManager` / `run_claude_session()`:**
- Read stdout line-by-line, parse each JSON event
- Store Claude's `session_id` (from the `init` event) in the DB for resuming
- Broadcast each `assistant` message chunk to WebSocket in real-time
- On `result` event, mark session completed

**New DB fields for `agent_sessions`:**
- `claude_session_id TEXT` — Claude CLI's own session ID (for `--resume`)

**Wire agent events to WebSocket:**
- In `run_server()`, spawn a task that subscribes to `agent_manager.subscribe()`
  and forwards `AgentEvent` → `WsEvent` on the main `ws_tx` channel

### 4.2 — Follow-up Messages (Backend)

**New endpoint: `POST /api/agents/{id}/messages`**
- Request body: `{ "content": "follow-up text" }`
- Looks up the session's `claude_session_id`
- Spawns `claude --resume <claude_session_id> --print --output-format stream-json --verbose "follow-up"`
- Streams output the same way as 4.1
- Stores messages in `agent_messages` table

### 4.3 — Agent Chat UI (Frontend)

**New page: `/agents/[id]`**
- Chat-style interface showing message history (user prompts + assistant responses)
- Messages stream in real-time via WebSocket
- Input box at the bottom to send follow-up messages
- Status indicator (running/completed/failed)
- Back link to agents list

**Update `/agents` page:**
- Make agent session rows clickable → navigate to `/agents/[id]`

### 4.4 — Porter MCP Server (TypeScript)

Build a TypeScript MCP server at `tools/porter-mcp/` using `@modelcontextprotocol/sdk`:

**Tools exposed:**
- `list_tasks(status?)` — query tasks via Porter API
- `create_task(title, description?, priority?)` — create a task
- `update_task(id, status?, title?, priority?)` — update a task
- `get_notifications(unread?)` — read notifications
- `integration_action(integration_id, action_name, params)` — call `handle()` on an integration

**Auto-configured:** Porter server automatically adds this MCP server to the config
for every agent session (pointing at `localhost:<port>`).

**Config in home.toml:**
```toml
[agents.mcp.porter]
command = "node"
args = ["tools/porter-mcp/dist/index.js"]
env = { PORTER_API_URL = "http://localhost:3101" }
```

### Execution order: 4.1 → 4.2 → 4.3 → 4.4

---

## Phase 5: MCP Integration Verification

### 5.1 — Simple MCP Server ✅ DONE
Added `@anthropic/mcp-server-fetch` to `config/home.toml`.

### 5.2 — Verify End-to-End
After Phase 4.3 (chat UI), test that:
- Start an agent: "Fetch https://httpbin.org/get and summarize what you see"
- See streaming output in the chat UI
- Send a follow-up message
- Confirm messages persist and are visible on page reload

---

## Phase 6: Google Calendar Integration

**Goal**: Built-in integration that syncs with Google Calendar via Gmail/Google API.

### 6.1 — OAuth2 Flow
- Add OAuth2 support to Porter (likely using the `oauth2` crate)
- Store tokens in `integrations_state` table (encrypted)
- Add `/api/integrations/calendar/auth` endpoint to initiate OAuth flow
- Handle callback to exchange code for tokens

### 6.2 — Calendar Integration Implementation
Create `crates/porter-integrations/src/calendar/mod.rs`:
- Implements `Integration` trait
- `init()` — loads stored OAuth tokens from DB
- `tick()` — polls Google Calendar API for upcoming events, returns notifications for events starting soon
- `handle_webhook()` — receives Google Calendar push notifications (if configured)
- `handle()` actions: `list_events`, `create_event`, `get_event`
- `capabilities()` — exposes calendar actions

### 6.3 — Config
```toml
[integrations]
enabled = ["tasks", "calendar"]

[integrations.calendar]
tick_interval = 300
calendar_id = "primary"
client_id = "env:GOOGLE_CLIENT_ID"
client_secret = "env:GOOGLE_CLIENT_SECRET"
```

### 6.4 — Frontend
- Calendar widget on dashboard showing today's events
- Integration settings page for connecting Google account

---

## Phase 7: Search & Filtering

**Goal**: Add search/filter to list views so users can quickly find tasks and agent sessions.

### 7.1 — Tasks Search
- Add a search input to the `/tasks` page header
- Filter tasks client-side by title, description, and tags
- Optional: add status/priority filter dropdowns alongside the search

### 7.2 — Agents Search
- Add a search input to the `/agents` page header
- Filter agent sessions by prompt text, session ID prefix, and status
- Optional: sort by date or status

---

## Phase 8: Polish & Hardening

- Graceful shutdown for tick loops (tokio CancellationToken)
- Better error handling for missing env vars (warn on startup)
- Integration health checks / status reporting
- Rate limiting on webhook endpoint
- Update INTEGRATIONS.md to reflect current state
- Tests for integration registration, tick loop, webhook handling

---

## Next Up

**Phase 4.1** — Switch agent sessions to streaming JSON output, wire events to WebSocket.
