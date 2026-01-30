"use client";

import { useMutation, useQueryClient } from "@tanstack/react-query";
import { api, Task } from "@/lib/api";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Button } from "@/components/ui/button";
import {
  Circle,
  CircleDot,
  CheckCircle2,
  XCircle,
  MoreHorizontal,
  Trash2,
} from "lucide-react";
import { cn } from "@/lib/utils";

const statusConfig: Record<
  Task["status"],
  { label: string; icon: typeof Circle; className: string }
> = {
  pending: {
    label: "Pending",
    icon: Circle,
    className: "text-yellow-500",
  },
  in_progress: {
    label: "In Progress",
    icon: CircleDot,
    className: "text-blue-500",
  },
  completed: {
    label: "Completed",
    icon: CheckCircle2,
    className: "text-green-500",
  },
  cancelled: {
    label: "Cancelled",
    icon: XCircle,
    className: "text-muted-foreground",
  },
};

const priorityConfig: Record<
  Task["priority"],
  { label: string; className: string }
> = {
  low: { label: "Low", className: "text-muted-foreground" },
  medium: { label: "Med", className: "text-yellow-600 dark:text-yellow-400" },
  high: { label: "High", className: "text-orange-600 dark:text-orange-400" },
  urgent: { label: "Urgent", className: "text-red-600 dark:text-red-400" },
};

const allStatuses: Task["status"][] = [
  "pending",
  "in_progress",
  "completed",
  "cancelled",
];

export function TaskCard({ task }: { task: Task }) {
  const queryClient = useQueryClient();
  const status = statusConfig[task.status];
  const priority = priorityConfig[task.priority];
  const StatusIcon = status.icon;

  const updateMutation = useMutation({
    mutationFn: (newStatus: Task["status"]) =>
      api.updateTask(task.id, { status: newStatus }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["tasks"] });
      queryClient.invalidateQueries({ queryKey: ["server-status"] });
    },
  });

  const deleteMutation = useMutation({
    mutationFn: () => api.deleteTask(task.id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["tasks"] });
      queryClient.invalidateQueries({ queryKey: ["server-status"] });
    },
  });

  const cycleStatus = () => {
    const next =
      task.status === "pending"
        ? "in_progress"
        : task.status === "in_progress"
          ? "completed"
          : null;
    if (next) updateMutation.mutate(next);
  };

  return (
    <Card
      className={cn(
        "transition-opacity",
        task.status === "completed" && "opacity-60",
        task.status === "cancelled" && "opacity-40"
      )}
    >
      <CardHeader className="pb-2">
        <div className="flex items-center gap-3">
          <button
            onClick={cycleStatus}
            className="shrink-0 hover:scale-110 transition-transform"
            title={`Status: ${status.label}. Click to advance.`}
          >
            <StatusIcon className={cn("h-5 w-5", status.className)} />
          </button>
          <CardTitle
            className={cn(
              "flex-1 text-base",
              task.status === "completed" && "line-through"
            )}
          >
            {task.title}
          </CardTitle>
          <Badge variant="outline" className={cn("text-xs", priority.className)}>
            {priority.label}
          </Badge>
          <DropdownMenu>
            <DropdownMenuTrigger asChild>
              <Button variant="ghost" size="icon" className="h-8 w-8">
                <MoreHorizontal className="h-4 w-4" />
              </Button>
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end">
              {allStatuses.map((s) => {
                const cfg = statusConfig[s];
                const Icon = cfg.icon;
                return (
                  <DropdownMenuItem
                    key={s}
                    onClick={() => updateMutation.mutate(s)}
                    disabled={s === task.status}
                  >
                    <Icon className={cn("h-4 w-4 mr-2", cfg.className)} />
                    {cfg.label}
                  </DropdownMenuItem>
                );
              })}
              <DropdownMenuSeparator />
              <DropdownMenuItem
                onClick={() => deleteMutation.mutate()}
                className="text-destructive"
              >
                <Trash2 className="h-4 w-4 mr-2" />
                Delete
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        </div>
      </CardHeader>
      {task.description && (
        <CardContent className="pl-11">
          <p className="text-sm text-muted-foreground">{task.description}</p>
        </CardContent>
      )}
    </Card>
  );
}
