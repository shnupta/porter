"use client";

import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { ThemeProvider } from "next-themes";
import { useState } from "react";
import { useWebSocket } from "@/hooks/use-websocket";

function WebSocketConnector({ children }: { children: React.ReactNode }) {
  useWebSocket();
  return <>{children}</>;
}

export function Providers({ children }: { children: React.ReactNode }) {
  const [queryClient] = useState(
    () =>
      new QueryClient({
        defaultOptions: {
          queries: {
            staleTime: 5000,
            retry: 1,
          },
        },
      })
  );

  return (
    <ThemeProvider attribute="class" defaultTheme="system" enableSystem>
      <QueryClientProvider client={queryClient}>
        <WebSocketConnector>{children}</WebSocketConnector>
      </QueryClientProvider>
    </ThemeProvider>
  );
}
