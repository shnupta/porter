"use client";

import { useEffect } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { wsClient, type WsEvent } from "@/lib/ws";

export function useWebSocket() {
  const queryClient = useQueryClient();

  useEffect(() => {
    wsClient.connect();

    const unsubscribe = wsClient.subscribe((event) => {
      switch (event.type) {
        case "TaskCreated":
        case "TaskUpdated":
        case "TaskDeleted":
          queryClient.invalidateQueries({ queryKey: ["tasks"] });
          queryClient.invalidateQueries({ queryKey: ["server-status"] });
          break;
        case "AgentOutput":
        case "AgentStatusChanged":
          queryClient.invalidateQueries({ queryKey: ["agent-sessions"] });
          queryClient.invalidateQueries({ queryKey: ["server-status"] });
          // Also invalidate the specific session's messages and detail
          if (event.data && typeof event.data === "object") {
            const data = event.data as { session_id?: string };
            if (data.session_id) {
              queryClient.invalidateQueries({
                queryKey: ["agent-session", data.session_id],
              });
              queryClient.invalidateQueries({
                queryKey: ["agent-messages", data.session_id],
              });
            }
          }
          break;
        case "Notification":
          queryClient.invalidateQueries({ queryKey: ["notifications"] });
          break;
      }
    });

    return () => {
      unsubscribe();
      wsClient.disconnect();
    };
  }, [queryClient]);
}

/**
 * Subscribe to streaming AgentOutput events for a specific session.
 * Calls `onChunk` with each text chunk as it arrives.
 */
export function useAgentStream(
  sessionId: string | undefined,
  onChunk: (content: string, contentType: string) => void
) {
  useEffect(() => {
    if (!sessionId) return;

    wsClient.connect();

    const unsubscribe = wsClient.subscribe((event: WsEvent) => {
      if (event.type === "AgentOutput" && event.data) {
        const data = event.data as {
          session_id?: string;
          content?: string;
          content_type?: string;
        };
        if (data.session_id === sessionId && data.content) {
          onChunk(data.content, data.content_type ?? "text");
        }
      }
    });

    return unsubscribe;
    // onChunk is intentionally excluded — callers should use a ref-stable callback
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [sessionId]);
}
