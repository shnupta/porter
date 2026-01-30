"use client";

import { useQuery } from "@tanstack/react-query";
import { api, Task } from "@/lib/api";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";

const statusColors: Record<Task["status"], string> = {
  pending: "bg-yellow-100 text-yellow-800",
  in_progress: "bg-blue-100 text-blue-800",
  completed: "bg-green-100 text-green-800",
  cancelled: "bg-gray-100 text-gray-800",
};

const priorityLabels: Record<Task["priority"], string> = {
  low: "Low",
  medium: "Med",
  high: "High",
  urgent: "Urgent",
};

export default function TasksPage() {
  const { data: tasks, isLoading } = useQuery({
    queryKey: ["tasks"],
    queryFn: () => api.listTasks(),
  });

  return (
    <div className="p-6 space-y-6">
      <div>
        <h1 className="text-2xl font-bold tracking-tight">Tasks</h1>
        <p className="text-muted-foreground">Manage your tasks</p>
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
      ) : tasks && tasks.length > 0 ? (
        <div className="space-y-3">
          {tasks.map((task) => (
            <Card key={task.id}>
              <CardHeader className="pb-2">
                <div className="flex items-center justify-between">
                  <CardTitle className="text-base">{task.title}</CardTitle>
                  <div className="flex gap-2">
                    <Badge variant="outline">
                      {priorityLabels[task.priority]}
                    </Badge>
                    <Badge className={statusColors[task.status]}>
                      {task.status.replace("_", " ")}
                    </Badge>
                  </div>
                </div>
              </CardHeader>
              {task.description && (
                <CardContent>
                  <p className="text-sm text-muted-foreground">
                    {task.description}
                  </p>
                </CardContent>
              )}
            </Card>
          ))}
        </div>
      ) : (
        <div className="rounded-lg border border-dashed p-8 text-center text-muted-foreground">
          No tasks yet. Create one from the CLI with: porter task new
          &quot;Your task&quot;
        </div>
      )}
    </div>
  );
}
