"use client";

import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { api, Task } from "@/lib/api";
import { CreateTaskDialog } from "@/components/tasks/create-task-dialog";
import { TaskCard } from "@/components/tasks/task-card";
import { cn } from "@/lib/utils";

type FilterTab = "all" | Task["status"];

const tabs: { value: FilterTab; label: string }[] = [
  { value: "all", label: "All" },
  { value: "pending", label: "Pending" },
  { value: "in_progress", label: "In Progress" },
  { value: "completed", label: "Completed" },
  { value: "cancelled", label: "Cancelled" },
];

export default function TasksPage() {
  const [filter, setFilter] = useState<FilterTab>("all");

  const { data: tasks, isLoading } = useQuery({
    queryKey: ["tasks"],
    queryFn: () => api.listTasks(),
  });

  const filtered =
    tasks && filter !== "all"
      ? tasks.filter((t) => t.status === filter)
      : tasks;

  const counts = tasks?.reduce(
    (acc, t) => {
      acc[t.status] = (acc[t.status] || 0) + 1;
      acc.all = (acc.all || 0) + 1;
      return acc;
    },
    {} as Record<string, number>
  );

  return (
    <div className="p-6 space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold tracking-tight">Tasks</h1>
          <p className="text-muted-foreground">Manage your tasks</p>
        </div>
        <CreateTaskDialog />
      </div>

      <div className="flex gap-1 border-b">
        {tabs.map((tab) => (
          <button
            key={tab.value}
            onClick={() => setFilter(tab.value)}
            className={cn(
              "px-3 py-2 text-sm font-medium border-b-2 transition-colors -mb-px",
              filter === tab.value
                ? "border-primary text-foreground"
                : "border-transparent text-muted-foreground hover:text-foreground"
            )}
          >
            {tab.label}
            {counts?.[tab.value] != null && (
              <span className="ml-1.5 text-xs text-muted-foreground">
                {counts[tab.value]}
              </span>
            )}
          </button>
        ))}
      </div>

      {isLoading ? (
        <div className="space-y-3">
          {[...Array(3)].map((_, i) => (
            <div
              key={i}
              className="h-20 rounded-lg border bg-card animate-pulse"
            />
          ))}
        </div>
      ) : filtered && filtered.length > 0 ? (
        <div className="space-y-2">
          {filtered.map((task) => (
            <TaskCard key={task.id} task={task} />
          ))}
        </div>
      ) : (
        <div className="rounded-lg border border-dashed p-8 text-center text-muted-foreground">
          {filter === "all"
            ? "No tasks yet. Click \"New Task\" to create one."
            : `No ${filter.replace("_", " ")} tasks.`}
        </div>
      )}
    </div>
  );
}
