const API_BASE = process.env.NEXT_PUBLIC_API_URL ?? "http://localhost:3101";

async function request<T>(path: string, options?: RequestInit): Promise<T> {
  const res = await fetch(`${API_BASE}${path}`, {
    headers: { "Content-Type": "application/json" },
    ...options,
  });

  if (!res.ok) {
    throw new Error(`API error: ${res.status} ${res.statusText}`);
  }

  if (res.status === 204) return undefined as T;
  return res.json();
}

// ── Types ──

export interface Task {
  id: string;
  title: string;
  description: string | null;
  status: "pending" | "in_progress" | "completed" | "cancelled";
  priority: "low" | "medium" | "high" | "urgent";
  tags: string[];
  due_date: string | null;
  integration_id: string | null;
  created_at: string;
  updated_at: string;
}

export interface CreateTask {
  title: string;
  description?: string;
  priority?: Task["priority"];
  tags?: string[];
  due_date?: string;
}

export interface UpdateTask {
  title?: string;
  description?: string;
  status?: Task["status"];
  priority?: Task["priority"];
  tags?: string[];
  due_date?: string;
}

export interface AgentSession {
  id: string;
  prompt: string;
  status: "running" | "paused" | "completed" | "failed";
  model: string;
  claude_session_id: string | null;
  working_directory: string | null;
  dangerously_skip_permissions: boolean;
  started_at: string;
  completed_at: string | null;
}

export interface AgentMessage {
  id: string;
  session_id: string;
  role: string;
  content: string;
  timestamp: string;
}

export interface IntegrationInfo {
  id: string;
  name: string;
  enabled: boolean;
  capabilities: string[];
}

export interface McpServerInfo {
  name: string;
  command: string;
}

export interface IntegrationsResponse {
  integrations: IntegrationInfo[];
  mcp_servers: McpServerInfo[];
}

export interface ServerStatus {
  instance_name: string;
  version: string;
  uptime_seconds: number;
  active_integrations: string[];
  mcp_servers: string[];
  active_agent_sessions: number;
  pending_tasks: number;
}

// ── API Functions ──

export const api = {
  // Health
  health: () => request<string>("/api/health"),
  status: () => request<ServerStatus>("/api/status"),

  // Tasks
  listTasks: (status?: string) =>
    request<Task[]>(`/api/tasks${status ? `?status=${status}` : ""}`),
  getTask: (id: string) => request<Task>(`/api/tasks/${id}`),
  createTask: (data: CreateTask) =>
    request<Task>("/api/tasks", {
      method: "POST",
      body: JSON.stringify(data),
    }),
  updateTask: (id: string, data: UpdateTask) =>
    request<Task>(`/api/tasks/${id}`, {
      method: "PUT",
      body: JSON.stringify(data),
    }),
  deleteTask: (id: string) =>
    request<void>(`/api/tasks/${id}`, { method: "DELETE" }),

  // Agents
  listSessions: (status?: string) =>
    request<AgentSession[]>(
      `/api/agents${status ? `?status=${status}` : ""}`
    ),
  getSession: (id: string) => request<AgentSession>(`/api/agents/${id}`),
  startSession: (
    prompt: string,
    opts?: { directory?: string; dangerously_skip_permissions?: boolean }
  ) =>
    request<AgentSession>("/api/agents", {
      method: "POST",
      body: JSON.stringify({ prompt, ...opts }),
    }),
  getMessages: (sessionId: string) =>
    request<AgentMessage[]>(`/api/agents/${sessionId}/messages`),
  sendMessage: (sessionId: string, content: string) =>
    request<AgentMessage>(`/api/agents/${sessionId}/messages`, {
      method: "POST",
      body: JSON.stringify({ content }),
    }),

  // Integrations
  listIntegrations: () => request<IntegrationsResponse>("/api/integrations"),
};
