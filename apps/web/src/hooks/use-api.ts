"use client";

import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { api } from "@/lib/api";
import {
  DecisionStats,
  StoredDecision,
  AuditVerificationResult,
  AuditEvent,
  ModelInfo,
  DecisionOutcome,
  RawTransactionPayload,
} from "@/types";

// ── Real-Time Decision Metrics ──────────────────────────────────────
export function useDecisionStats() {
  return useQuery({
    queryKey: ["decision-stats"],
    queryFn: () => api.get<DecisionStats>("/decisions/stats"),
    refetchInterval: 10000,
  });
}

// ── Stream of Evaluated Decisions ───────────────────────────────────
export function useRecentDecisions(status?: string, limit = 50) {
  return useQuery({
    queryKey: ["recent-decisions", status, limit],
    queryFn: () => {
      const q = status && status !== "ALL" ? `?status=${status}&limit=${limit}` : `?limit=${limit}`;
      return api.get<StoredDecision[]>(`/decisions${q}`);
    },
    refetchInterval: 10000,
  });
}

// ── Single Decision Detail ──────────────────────────────────────────
export function useDecisionDetail(id?: string) {
  return useQuery({
    queryKey: ["decision", id],
    queryFn: () => api.get<StoredDecision>(`/decisions/${id}`),
    enabled: !!id,
  });
}

// ── Cryptographic Integrity Probes ──────────────────────────────────
export function useAuditVerification() {
  return useQuery({
    queryKey: ["audit-verify"],
    queryFn: () => api.get<AuditVerificationResult>("/audit/verify"),
  });
}

export function useAuditRecords() {
  return useQuery({
    queryKey: ["audit-records"],
    queryFn: () => api.get<AuditEvent[]>("/audit"),
  });
}

// ── ML Model Spec ───────────────────────────────────────────────────
export function useModelInfo() {
  return useQuery({
    queryKey: ["model-info"],
    queryFn: () => api.get<ModelInfo>("/model"),
  });
}

// ── Live Simulation Mutation ────────────────────────────────────────
export function useSimulateTransaction() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (payload: RawTransactionPayload) =>
      api.post<DecisionOutcome>("/transactions", payload),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["decision-stats"] });
      queryClient.invalidateQueries({ queryKey: ["recent-decisions"] });
    },
  });
}
