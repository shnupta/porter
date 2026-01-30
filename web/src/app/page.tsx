"use client";

import { CheckSquare, Bot, Puzzle, Clock } from "lucide-react";
import { StatusCard } from "@/components/dashboard/status-card";
import { useServerStatus } from "@/hooks/use-server-status";
import { Badge } from "@/components/ui/badge";

function formatUptime(seconds: number): string {
  const h = Math.floor(seconds / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  if (h > 0) return `${h}h ${m}m`;
  return `${m}m`;
}

export default function DashboardPage() {
  const { data: status, isLoading, isError } = useServerStatus();

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
            title="Active Skills"
            value={status.active_skills.length}
            description={status.active_skills.join(", ") || "None"}
            icon={Puzzle}
          />
          <StatusCard
            title="Uptime"
            value={formatUptime(status.uptime_seconds)}
            icon={Clock}
          />
        </div>
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
