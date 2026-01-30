"use client";

import { useParams, useRouter } from "next/navigation";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { useCallback, useEffect, useRef, useState } from "react";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import {
  AlertCircle,
  ArrowLeft,
  Send,
  Loader2,
  Bot,
  User,
  FolderOpen,
  Wrench,
  Square,
} from "lucide-react";
import { api, type AgentMessage } from "@/lib/api";
import { useAgentStream } from "@/hooks/use-websocket";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Textarea } from "@/components/ui/textarea";
import { cn } from "@/lib/utils";

export default function AgentSessionPage() {
  const { id } = useParams<{ id: string }>();
  const router = useRouter();
  const queryClient = useQueryClient();
  const scrollRef = useRef<HTMLDivElement>(null);
  const [input, setInput] = useState("");

  interface StreamBlock {
    type: "text" | "thinking" | "tool_use";
    content: string;
  }
  const [streamBlocks, setStreamBlocks] = useState<StreamBlock[]>([]);

  const { data: session, isLoading: sessionLoading } = useQuery({
    queryKey: ["agent-session", id],
    queryFn: () => api.getSession(id),
    refetchInterval: (query) => {
      const status = query.state.data?.status;
      return status === "running" ? 2000 : false;
    },
  });

  const { data: messages, isLoading: messagesLoading } = useQuery({
    queryKey: ["agent-messages", id],
    queryFn: () => api.getMessages(id),
  });

  // When messages refetch (after stream completes), clear streaming blocks
  useEffect(() => {
    if (session?.status !== "running") {
      setStreamBlocks([]);
    }
  }, [session?.status, messages]);

  // Stream incoming chunks into local state
  const onChunk = useCallback(
    (content: string, contentType: string) => {
      const type = (contentType === "thinking" || contentType === "tool_use"
        ? contentType
        : "text") as StreamBlock["type"];

      setStreamBlocks((prev) => {
        const last = prev[prev.length - 1];
        if (last && last.type === type) {
          const updated = [...prev];
          updated[updated.length - 1] = {
            ...last,
            content: last.content + content,
          };
          return updated;
        }
        return [...prev, { type, content }];
      });
    },
    []
  );

  useAgentStream(id, onChunk);

  // Auto-scroll to bottom
  useEffect(() => {
    scrollRef.current?.scrollTo({
      top: scrollRef.current.scrollHeight,
      behavior: "smooth",
    });
  }, [messages, streamBlocks]);

  const sendMutation = useMutation({
    mutationFn: (content: string) => api.sendMessage(id, content),
    onSuccess: () => {
      setInput("");
      setStreamBlocks([]);
      queryClient.invalidateQueries({ queryKey: ["agent-messages", id] });
      queryClient.invalidateQueries({ queryKey: ["agent-session", id] });
    },
  });

  const cancelMutation = useMutation({
    mutationFn: () => api.cancelSession(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["agent-session", id] });
      queryClient.invalidateQueries({ queryKey: ["agent-messages", id] });
    },
  });

  const handleSend = () => {
    const trimmed = input.trim();
    if (!trimmed || sendMutation.isPending) return;
    sendMutation.mutate(trimmed);
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  const isLoading = sessionLoading || messagesLoading;
  const canSend =
    session?.claude_session_id &&
    session.status !== "running" &&
    input.trim().length > 0;

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="flex items-center gap-3 border-b px-4 py-3">
        <Button
          variant="ghost"
          size="icon-sm"
          onClick={() => router.push("/agents")}
        >
          <ArrowLeft className="h-4 w-4" />
        </Button>
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2">
            <h1 className="text-sm font-semibold truncate font-mono">
              {id.slice(0, 8)}
            </h1>
            {session && (
              <>
                <Badge variant="outline" className="text-xs">
                  {session.model}
                </Badge>
                {session.working_directory && (
                  <Badge variant="outline" className="text-xs font-mono">
                    <FolderOpen className="h-3 w-3 mr-1" />
                    {session.working_directory.split("/").pop()}
                  </Badge>
                )}
                <Badge
                  variant={
                    session.status === "running" ? "default" : "secondary"
                  }
                  className="text-xs"
                >
                  {session.status === "running" && (
                    <Loader2 className="h-3 w-3 animate-spin mr-1" />
                  )}
                  {session.status}
                </Badge>
              </>
            )}
          </div>
          {session && (
            <p className="text-xs text-muted-foreground truncate mt-0.5">
              {session.prompt}
            </p>
          )}
        </div>
      </div>

      {/* Messages */}
      <div ref={scrollRef} className="flex-1 overflow-y-auto p-4 space-y-4">
        {isLoading ? (
          <div className="space-y-3">
            {[...Array(3)].map((_, i) => (
              <div
                key={i}
                className="h-16 rounded-lg bg-muted/50 animate-pulse"
              />
            ))}
          </div>
        ) : messages && messages.length > 0 ? (
          <>
            {messages.map((msg) => (
              <MessageBubble key={msg.id} message={msg} />
            ))}
            {/* Streaming blocks that haven't been persisted yet */}
            {streamBlocks.length > 0 && session?.status === "running" && (
              <div className="flex gap-3">
                <div className="flex h-7 w-7 shrink-0 items-center justify-center rounded-full bg-primary/10 text-primary">
                  <Bot className="h-4 w-4" />
                </div>
                <div className="flex-1 space-y-2">
                  {streamBlocks.map((block, i) => {
                    const isLast = i === streamBlocks.length - 1;
                    if (block.type === "thinking") {
                      return (
                        <details
                          key={i}
                          open={isLast}
                          className="rounded-lg bg-muted/30 border border-border/50"
                        >
                          <summary className="px-3 py-1.5 text-xs text-muted-foreground cursor-pointer select-none">
                            Thinking...
                          </summary>
                          <div className="px-3 pb-2 text-sm italic text-muted-foreground/70 whitespace-pre-wrap">
                            {block.content}
                            {isLast && (
                              <span className="inline-block w-1.5 h-4 bg-muted-foreground/40 animate-pulse ml-0.5 align-text-bottom" />
                            )}
                          </div>
                        </details>
                      );
                    }
                    if (block.type === "tool_use") {
                      return (
                        <div
                          key={i}
                          className="inline-flex items-center gap-1.5 rounded-md bg-muted/50 border border-border/50 px-2.5 py-1 text-xs font-mono text-muted-foreground"
                        >
                          <Wrench className="h-3 w-3" />
                          {block.content}
                        </div>
                      );
                    }
                    return (
                      <div
                        key={i}
                        className="rounded-lg bg-muted/50 px-3 py-2 text-sm prose prose-sm dark:prose-invert max-w-none"
                      >
                        <ReactMarkdown remarkPlugins={[remarkGfm]}>
                          {block.content}
                        </ReactMarkdown>
                        {isLast && (
                          <span className="inline-block w-1.5 h-4 bg-foreground/70 animate-pulse ml-0.5 align-text-bottom" />
                        )}
                      </div>
                    );
                  })}
                </div>
              </div>
            )}
          </>
        ) : (
          <div className="text-center text-sm text-muted-foreground pt-8">
            Waiting for response...
          </div>
        )}
      </div>

      {/* Input */}
      <div className="border-t p-4">
        <div className="flex gap-2">
          <Textarea
            value={input}
            onChange={(e) => setInput(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder={
              !session?.claude_session_id
                ? "Waiting for session to initialize..."
                : session.status === "running"
                  ? "Agent is responding..."
                  : "Send a follow-up message..."
            }
            disabled={!session?.claude_session_id || session.status === "running"}
            className="min-h-10 max-h-32 resize-none"
            rows={1}
          />
          {session?.status === "running" ? (
            <Button
              variant="outline"
              size="icon"
              onClick={() => cancelMutation.mutate()}
              disabled={cancelMutation.isPending}
              title="Stop agent"
            >
              {cancelMutation.isPending ? (
                <Loader2 className="h-4 w-4 animate-spin" />
              ) : (
                <Square className="h-4 w-4" />
              )}
            </Button>
          ) : (
            <Button
              size="icon"
              onClick={handleSend}
              disabled={!canSend || sendMutation.isPending}
            >
              {sendMutation.isPending ? (
                <Loader2 className="h-4 w-4 animate-spin" />
              ) : (
                <Send className="h-4 w-4" />
              )}
            </Button>
          )}
        </div>
      </div>
    </div>
  );
}

function MessageBubble({ message }: { message: AgentMessage }) {
  const isUser = message.role === "user";
  const isError = message.role === "error";

  return (
    <div className="flex gap-3">
      <div
        className={cn(
          "flex h-7 w-7 shrink-0 items-center justify-center rounded-full",
          isError
            ? "bg-destructive/10 text-destructive"
            : isUser
              ? "bg-secondary text-secondary-foreground"
              : "bg-primary/10 text-primary"
        )}
      >
        {isError ? (
          <AlertCircle className="h-4 w-4" />
        ) : isUser ? (
          <User className="h-4 w-4" />
        ) : (
          <Bot className="h-4 w-4" />
        )}
      </div>
      <div
        className={cn(
          "flex-1 rounded-lg px-3 py-2 text-sm",
          isError
            ? "bg-destructive/10 text-destructive border border-destructive/20"
            : isUser
              ? "bg-secondary/50 whitespace-pre-wrap"
              : "bg-muted/50 prose prose-sm dark:prose-invert max-w-none"
        )}
      >
        {isUser ? (
          message.content
        ) : isError ? (
          <span className="font-medium">{message.content}</span>
        ) : (
          <ReactMarkdown remarkPlugins={[remarkGfm]}>
            {message.content}
          </ReactMarkdown>
        )}
      </div>
    </div>
  );
}
