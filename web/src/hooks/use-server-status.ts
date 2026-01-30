"use client";

import { useQuery } from "@tanstack/react-query";
import { api } from "@/lib/api";

export function useServerStatus() {
  return useQuery({
    queryKey: ["server-status"],
    queryFn: api.status,
    refetchInterval: 30000,
  });
}
