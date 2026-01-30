"use client";

import Link from "next/link";
import { useQuery } from "@tanstack/react-query";
import { CheckSquare, Bot, Puzzle, Clock, ArrowRight } from "lucide-react";
import { StatusCard } from "@/components/dashboard/status-card";
import { useServerStatus } from "@/hooks/use-server-status";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { api, Task } from "@/lib/api";
import { cn } from "@/lib/utils";

function formatUptime(seconds: number): string {
  const h = Math.floor(seconds / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  if (h > 0) return `${h}h ${m}m`;
  return `${m}m`;
}

const statusIcon: Record<Task["status"], string> = {
  pending: "text-yellow-500",
  in_progress: "text-blue-500",
  completed: "text-green-500",
  cancelled: "text-muted-foreground",
};

export default function DashboardPage() {
  const { data: status, isLoading, isError } = useServerStatus();

  const { data: tasks } = useQuery({
    queryKey: ["tasks"],
    queryFn: () => api.listTasks(),
  });

  const recentTasks = tasks?.slice(0, 5);
  const tasksByStatus = tasks?.reduce(
    (acc, t) => {
      acc[t.status] = (acc[t.status] || 0) + 1;
      return acc;
    },
    {} as Record<string, number>
  );

  return (
    <div className="p-6 space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold tracking-tight">Dashboard</h1>
          <p className="text-muted-foreground">
            {status
              ? `${status.instance_name} v${status.version}`
              : "Connecting..."}
          </p>
        </div>
        {status && (
          <Badge variant="secondary" className="text-xs">
            Online
          </Badge>
        )}
        {isError && (
          <Badge variant="destructive" className="text-xs">
            Offline
          </Badge>
        )}
      </div>

      {isLoading ? (
        <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
          {[...Array(4)].map((_, i) => (
            <div
              key={i}
              className="h-[120px] rounded-lg border bg-card animate-pulse"
            />
          ))}
        </div>
      ) : status ? (
        <>
          <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
            <StatusCard
              title="Pending Tasks"
              value={status.pending_tasks}
              icon={CheckSquare}
            />
            <StatusCard
              title="Active Agents"
              value={status.active_agent_sessions}
              icon={Bot}
            />
            <StatusCard
              title="Integrations"
              value={status.active_integrations.length + status.mcp_servers.length}
              description={
                [...status.active_integrations, ...status.mcp_servers].join(", ") || "None"
              }
              icon={Puzzle}
            />
            <StatusCard
              title="Uptime"
              value={formatUptime(status.uptime_seconds)}
              icon={Clock}
            />
          </div>

          <div className="grid gap-4 md:grid-cols-2">
            <Card>
              <CardHeader className="flex flex-row items-center justify-between pb-2">
                <CardTitle className="text-sm font-medium">
                  Recent Tasks
                </CardTitle>
                <Link
                  href="/tasks"
                  className="text-xs text-muted-foreground hover:text-foreground flex items-center gap-1"
                >
                  View all <ArrowRight className="h-3 w-3" />
                </Link>
              </CardHeader>
              <CardContent>
                {recentTasks && recentTasks.length > 0 ? (
                  <div className="space-y-3">
                    {recentTasks.map((task) => (
                      <div
                        key={task.id}
                        className="flex items-center justify-between"
                      >
                        <div className="flex items-center gap-2 min-w-0">
                          <div
                            className={cn(
                              "h-2 w-2 rounded-full shrink-0",
                              statusIcon[task.status].replace("text-", "bg-")
                            )}
                          />
                          <span
                            className={cn(
                              "text-sm truncate",
                              task.status === "completed" &&
                                "line-through text-muted-foreground"
                            )}
                          >
                            {task.title}
                          </span>
                        </div>
                        <Badge variant="outline" className="text-xs shrink-0 ml-2">
                          {task.priority}
                        </Badge>
                      </div>
                    ))}
                  </div>
                ) : (
                  <p className="text-sm text-muted-foreground">
                    No tasks yet.
                  </p>
                )}
              </CardContent>
            </Card>

            <Card>
              <CardHeader className="pb-2">
                <CardTitle className="text-sm font-medium">
                  Task Breakdown
                </CardTitle>
              </CardHeader>
              <CardContent>
                {tasksByStatus && Object.keys(tasksByStatus).length > 0 ? (
                  <div className="space-y-3">
                    {(
                      [
                        ["pending", "Pending", "bg-yellow-500"],
                        ["in_progress", "In Progress", "bg-blue-500"],
                        ["completed", "Completed", "bg-green-500"],
                        ["cancelled", "Cancelled", "bg-gray-400"],
                      ] as const
                    ).map(([key, label, color]) => {
                      const count = tasksByStatus[key] || 0;
                      const total = tasks?.length || 1;
                      const pct = Math.round((count / total) * 100);
                      return (
                        <div key={key} className="space-y-1">
                          <div className="flex items-center justify-between text-sm">
                            <span>{label}</span>
                            <span className="text-muted-foreground">
                              {count}
                            </span>
                          </div>
                          <div className="h-2 rounded-full bg-muted overflow-hidden">
                            <div
                              className={cn("h-full rounded-full", color)}
                              style={{ width: `${pct}%` }}
                            />
                          </div>
                        </div>
                      );
                    })}
                  </div>
                ) : (
                  <p className="text-sm text-muted-foreground">
                    No tasks yet.
                  </p>
                )}
              </CardContent>
            </Card>
          </div>
        </>
      ) : (
        <div className="rounded-lg border border-dashed p-8 text-center text-muted-foreground">
          <p>Unable to connect to the Porter server.</p>
          <p className="text-sm mt-1">
            Start it with: porter serve --config config/home.toml
          </p>
        </div>
      )}
    </div>
  );
}
