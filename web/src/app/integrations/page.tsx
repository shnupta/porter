"use client";

import { useQuery } from "@tanstack/react-query";
import { api } from "@/lib/api";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";

export default function IntegrationsPage() {
  const { data, isLoading } = useQuery({
    queryKey: ["integrations"],
    queryFn: api.listIntegrations,
  });

  return (
    <div className="p-6 space-y-6">
      <div>
        <h1 className="text-2xl font-bold tracking-tight">Integrations</h1>
        <p className="text-muted-foreground">
          Built-in integrations and MCP servers available to agents
        </p>
      </div>

      {isLoading ? (
        <div className="grid gap-4 md:grid-cols-2">
          {[...Array(2)].map((_, i) => (
            <div
              key={i}
              className="h-32 rounded-lg border bg-card animate-pulse"
            />
          ))}
        </div>
      ) : (
        <>
          {data && data.integrations.length > 0 && (
            <div className="space-y-3">
              <h2 className="text-sm font-medium text-muted-foreground uppercase tracking-wider">
                Built-in Integrations
              </h2>
              <div className="grid gap-4 md:grid-cols-2">
                {data.integrations.map((integration) => (
                  <Card key={integration.id}>
                    <CardHeader className="pb-2">
                      <div className="flex items-center justify-between">
                        <CardTitle className="text-base">
                          {integration.name}
                        </CardTitle>
                        <Badge
                          variant={
                            integration.enabled ? "default" : "secondary"
                          }
                        >
                          {integration.enabled ? "Active" : "Disabled"}
                        </Badge>
                      </div>
                    </CardHeader>
                    <CardContent>
                      <div className="flex flex-wrap gap-1">
                        {integration.capabilities.map((cap) => (
                          <Badge
                            key={cap}
                            variant="outline"
                            className="text-xs"
                          >
                            {cap}
                          </Badge>
                        ))}
                      </div>
                    </CardContent>
                  </Card>
                ))}
              </div>
            </div>
          )}

          {data && data.mcp_servers.length > 0 && (
            <div className="space-y-3">
              <h2 className="text-sm font-medium text-muted-foreground uppercase tracking-wider">
                MCP Servers
              </h2>
              <div className="grid gap-4 md:grid-cols-2">
                {data.mcp_servers.map((server) => (
                  <Card key={server.name}>
                    <CardHeader className="pb-2">
                      <div className="flex items-center justify-between">
                        <CardTitle className="text-base">
                          {server.name}
                        </CardTitle>
                        <Badge variant="secondary">MCP</Badge>
                      </div>
                    </CardHeader>
                    <CardContent>
                      <code className="text-xs text-muted-foreground">
                        {server.command}
                      </code>
                    </CardContent>
                  </Card>
                ))}
              </div>
            </div>
          )}

          {data &&
            data.integrations.length === 0 &&
            data.mcp_servers.length === 0 && (
              <div className="rounded-lg border border-dashed p-8 text-center text-muted-foreground">
                No integrations configured. Enable integrations or add MCP
                servers in your config file.
              </div>
            )}
        </>
      )}
    </div>
  );
}
