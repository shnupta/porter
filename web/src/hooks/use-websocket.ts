"use client";

import { useEffect } from "react";
import { useQueryClient } from "@tanstack/react-query";
import { wsClient } from "@/lib/ws";

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
