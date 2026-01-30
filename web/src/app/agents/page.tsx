"use client";

import { useState } from "react";
import { useRouter } from "next/navigation";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { api } from "@/lib/api";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Textarea } from "@/components/ui/textarea";
import { Plus, Loader2, ChevronRight, FolderOpen, Trash2 } from "lucide-react";

export default function AgentsPage() {
  const router = useRouter();
  const queryClient = useQueryClient();
  const [dialogOpen, setDialogOpen] = useState(false);
  const [prompt, setPrompt] = useState("");
  const [directory, setDirectory] = useState("");
  const [skipPermissions, setSkipPermissions] = useState(false);

  const { data: sessions, isLoading } = useQuery({
    queryKey: ["agent-sessions"],
    queryFn: () => api.listSessions(),
  });

  const deleteMutation = useMutation({
    mutationFn: (id: string) => api.deleteSession(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["agent-sessions"] });
    },
  });

  const startMutation = useMutation({
    mutationFn: () =>
      api.startSession(prompt, {
        directory: directory.trim() || undefined,
        dangerously_skip_permissions: skipPermissions,
      }),
    onSuccess: (session) => {
      queryClient.invalidateQueries({ queryKey: ["agent-sessions"] });
      setDialogOpen(false);
      setPrompt("");
      setDirectory("");
      setSkipPermissions(false);
      router.push(`/agents/${session.id}`);
    },
  });

  const handleStart = () => {
    if (!prompt.trim() || startMutation.isPending) return;
    startMutation.mutate();
  };

  return (
    <div className="p-6 space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold tracking-tight">Agent Sessions</h1>
          <p className="text-muted-foreground">
            Manage Claude agent sessions
          </p>
        </div>

        <Dialog open={dialogOpen} onOpenChange={setDialogOpen}>
          <DialogTrigger asChild>
            <Button>
              <Plus className="h-4 w-4" />
              New Session
            </Button>
          </DialogTrigger>
          <DialogContent>
            <DialogHeader>
              <DialogTitle>Start Agent Session</DialogTitle>
            </DialogHeader>
            <div className="space-y-4 pt-2">
              <div className="space-y-2">
                <Label htmlFor="prompt">Prompt</Label>
                <Textarea
                  id="prompt"
                  value={prompt}
                  onChange={(e) => setPrompt(e.target.value)}
                  placeholder="What should the agent do?"
                  className="min-h-24"
                  onKeyDown={(e) => {
                    if (e.key === "Enter" && (e.metaKey || e.ctrlKey)) {
                      e.preventDefault();
                      handleStart();
                    }
                  }}
                />
              </div>

              <div className="space-y-2">
                <Label htmlFor="directory">
                  <FolderOpen className="inline h-3.5 w-3.5 mr-1" />
                  Project Directory
                </Label>
                <Input
                  id="directory"
                  value={directory}
                  onChange={(e) => setDirectory(e.target.value)}
                  placeholder="/path/to/project (optional)"
                />
                <p className="text-xs text-muted-foreground">
                  Leave empty to start without a project context.
                </p>
              </div>

              <div className="flex items-center gap-2">
                <input
                  type="checkbox"
                  id="skip-permissions"
                  checked={skipPermissions}
                  onChange={(e) => setSkipPermissions(e.target.checked)}
                  className="h-4 w-4 rounded border-input"
                />
                <Label
                  htmlFor="skip-permissions"
                  className="text-sm font-normal cursor-pointer"
                >
                  Skip permission prompts
                </Label>
              </div>

              <div className="flex justify-end">
                <Button
                  onClick={handleStart}
                  disabled={!prompt.trim() || startMutation.isPending}
                >
                  {startMutation.isPending && (
                    <Loader2 className="h-4 w-4 animate-spin" />
                  )}
                  Start
                </Button>
              </div>
            </div>
          </DialogContent>
        </Dialog>
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
            <Card
              key={session.id}
              className="group cursor-pointer transition-colors hover:bg-accent/50"
              onClick={() => router.push(`/agents/${session.id}`)}
            >
              <CardHeader className="pb-2">
                <div className="flex items-center justify-between">
                  <CardTitle className="text-base font-mono text-sm">
                    {session.id.slice(0, 8)}
                  </CardTitle>
                  <div className="flex items-center gap-2">
                    <Badge variant="outline">{session.model}</Badge>
                    {session.working_directory && (
                      <Badge variant="outline" className="font-mono text-xs">
                        <FolderOpen className="h-3 w-3 mr-1" />
                        {session.working_directory.split("/").pop()}
                      </Badge>
                    )}
                    <Badge
                      variant={
                        session.status === "running" ? "default" : "secondary"
                      }
                    >
                      {session.status === "running" && (
                        <Loader2 className="h-3 w-3 animate-spin mr-1" />
                      )}
                      {session.status}
                    </Badge>
                    {session.status !== "running" && (
                      <Button
                        variant="ghost"
                        size="icon-sm"
                        className="opacity-0 group-hover:opacity-100 transition-opacity text-muted-foreground hover:text-destructive"
                        onClick={(e) => {
                          e.stopPropagation();
                          deleteMutation.mutate(session.id);
                        }}
                      >
                        <Trash2 className="h-3.5 w-3.5" />
                      </Button>
                    )}
                    <ChevronRight className="h-4 w-4 text-muted-foreground" />
                  </div>
                </div>
              </CardHeader>
              <CardContent>
                <p className="text-sm text-muted-foreground line-clamp-2">
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
          <p>No agent sessions yet.</p>
          <Button
            variant="outline"
            className="mt-4"
            onClick={() => setDialogOpen(true)}
          >
            <Plus className="h-4 w-4" />
            Start your first session
          </Button>
        </div>
      )}
    </div>
  );
}
