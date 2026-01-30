"use client";

import { useQuery } from "@tanstack/react-query";
import { api } from "@/lib/api";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";

export default function AgentsPage() {
  const { data: sessions, isLoading } = useQuery({
    queryKey: ["agent-sessions"],
    queryFn: () => api.listSessions(),
  });

  return (
    <div className="p-6 space-y-6">
      <div>
        <h1 className="text-2xl font-bold tracking-tight">Agent Sessions</h1>
        <p className="text-muted-foreground">Manage Claude agent sessions</p>
      </div>

      {isLoading ? (
        <div className="space-y-3">
          {[...Array(2)].map((_, i) => (
            <div
              key={i}
              className="h-24 rounded-lg border bg-card animate-pulse"
            />
          ))}
        </div>
      ) : sessions && sessions.length > 0 ? (
        <div className="space-y-3">
          {sessions.map((session) => (
            <Card key={session.id}>
              <CardHeader className="pb-2">
                <div className="flex items-center justify-between">
                  <CardTitle className="text-base font-mono text-sm">
                    {session.id.slice(0, 8)}
                  </CardTitle>
                  <div className="flex gap-2">
                    <Badge variant="outline">{session.model}</Badge>
                    <Badge
                      variant={
                        session.status === "running" ? "default" : "secondary"
                      }
                    >
                      {session.status}
                    </Badge>
                  </div>
                </div>
              </CardHeader>
              <CardContent>
                <p className="text-sm text-muted-foreground">
                  {session.prompt}
                </p>
                <p className="text-xs text-muted-foreground mt-2">
                  Started: {new Date(session.started_at).toLocaleString()}
                </p>
              </CardContent>
            </Card>
          ))}
        </div>
      ) : (
        <div className="rounded-lg border border-dashed p-8 text-center text-muted-foreground">
          No agent sessions. Start one with: porter agent start &quot;Your
          prompt&quot;
        </div>
      )}
    </div>
  );
}
