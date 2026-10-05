"use client";

import { useEffect, useState } from "react";
import { api } from "@/lib/api";
import { AuditEvent, AuditVerificationResult } from "@/types";
import { DecisionBadge } from "@/components/shared/banking";
import {
  ShieldCheck,
  ShieldAlert,
  RotateCw,
  CheckCircle2,
  FileCheck2,
  Layers,
} from "lucide-react";
import { Button } from "@/components/ui/button";

export default function AuditPage() {
  const [events, setEvents] = useState<AuditEvent[]>([]);
  const [verification, setVerification] = useState<AuditVerificationResult | null>(null);
  const [loading, setLoading] = useState(true);
  const [verifying, setVerifying] = useState(false);

  const fetchAuditData = async () => {
    setLoading(true);
    try {
      const data = await api.get<AuditEvent[]>("/audit");
      setEvents(data);
    } catch (err) {
      console.error("Failed to load audit events", err);
    } finally {
      setLoading(false);
    }
  };

  const handleVerify = async () => {
    setVerifying(true);
    try {
      const result = await api.get<AuditVerificationResult>("/audit/verify");
      setVerification(result);
    } catch (err) {
      console.error("Failed to verify audit hash chain", err);
    } finally {
      setVerifying(false);
    }
  };

  useEffect(() => {
    fetchAuditData();
    handleVerify();
  }, []);

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <div className="flex items-center gap-2">
            <span className="rounded-md bg-primary/10 px-2 py-0.5 text-xs font-semibold text-primary">
              CRYPTOGRAPHIC LEDGER
            </span>
            <span className="text-xs text-muted-foreground font-mono">Tamper-Evident SHA-256</span>
          </div>
          <h1 className="text-2xl font-bold tracking-tight text-foreground mt-1">
            Audit Trail & Verification
          </h1>
          <p className="text-sm text-muted-foreground">
            Immutable append-only ledger tracking all transaction decisions, rule changes, and analyst actions.
          </p>
        </div>

        <div className="flex items-center gap-2">
          <Button
            onClick={handleVerify}
            disabled={verifying}
            className="gap-2 text-xs font-medium shadow-md"
          >
            {verifying ? (
              <RotateCw className="h-3.5 w-3.5 animate-spin" />
            ) : (
              <ShieldCheck className="h-3.5 w-3.5" />
            )}
            Verify Chain Integrity
          </Button>
          <Button
            variant="outline"
            size="sm"
            onClick={fetchAuditData}
            disabled={loading}
            className="gap-1 text-xs"
          >
            <RotateCw className={`h-3 w-3 ${loading ? "animate-spin" : ""}`} />
            Refresh
          </Button>
        </div>
      </div>

      {/* Verification Summary Banner */}
      {verification && (
        <div
          className={`rounded-xl border p-5 transition ${
            verification.valid
              ? "border-emerald-500/30 bg-emerald-500/5 text-emerald-400"
              : "border-red-500/30 bg-red-500/5 text-red-400"
          }`}
        >
          <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
            <div className="flex items-center gap-3">
              <div
                className={`flex h-10 w-10 items-center justify-center rounded-xl border ${
                  verification.valid
                    ? "border-emerald-500/40 bg-emerald-500/10 text-emerald-400"
                    : "border-red-500/40 bg-red-500/10 text-red-400"
                }`}
              >
                {verification.valid ? (
                  <CheckCircle2 className="h-5 w-5" />
                ) : (
                  <ShieldAlert className="h-5 w-5" />
                )}
              </div>
              <div>
                <h3 className="font-semibold text-sm text-foreground">
                  {verification.valid
                    ? "Cryptographic Hash Chain Verified Intact"
                    : "Tamper Detected in Hash Chain!"}
                </h3>
                <p className="text-xs text-muted-foreground font-mono">
                  Chain Status: {verification.chain_status} • Verified at: {new Date(verification.verified_at).toLocaleTimeString()}
                </p>
              </div>
            </div>

            <div className="font-mono text-xs text-right space-y-0.5">
              <div className="text-muted-foreground text-[10px] uppercase">Ledger State</div>
              <div className="font-semibold text-emerald-400">
                SEQUENCED & APPEND-ONLY
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Audit Events List */}
      <div className="rounded-xl border border-border/80 bg-card overflow-hidden shadow-sm">
        <div className="p-4 border-b border-border/80 flex items-center justify-between bg-muted/20">
          <div className="flex items-center gap-2 font-semibold text-xs text-foreground uppercase tracking-wider font-mono">
            <FileCheck2 className="h-4 w-4 text-primary" />
            Append-Only Audit Ledger
          </div>
          <span className="text-[11px] font-mono text-muted-foreground">
            {events.length} records in memory buffer
          </span>
        </div>

        <div className="overflow-x-auto">
          <table className="w-full text-left text-xs font-mono">
            <thead className="bg-muted/40 text-muted-foreground uppercase border-b border-border/80">
              <tr>
                <th className="px-4 py-3">Audit ID</th>
                <th className="px-4 py-3">Timestamp</th>
                <th className="px-4 py-3">Event UUID</th>
                <th className="px-4 py-3">Decision</th>
                <th className="px-4 py-3">Model</th>
                <th className="px-4 py-3">Reasons</th>
                <th className="px-4 py-3">Status</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-border/60">
              {events.length === 0 ? (
                <tr>
                  <td colSpan={7} className="px-4 py-12 text-center text-muted-foreground">
                    {loading ? "Reading tamper-evident log..." : "No audit events registered yet."}
                  </td>
                </tr>
              ) : (
                events.map((evt) => (
                  <tr key={evt.audit_id} className="hover:bg-muted/20 transition">
                    <td className="px-4 py-3 font-semibold text-foreground whitespace-nowrap">
                      {evt.audit_id.substring(0, 18)}...
                    </td>
                    <td className="px-4 py-3 text-muted-foreground whitespace-nowrap">
                      {new Date(evt.created_at).toLocaleString()}
                    </td>
                    <td className="px-4 py-3 text-foreground whitespace-nowrap font-mono">
                      {evt.event_id.substring(0, 14)}...
                    </td>
                    <td className="px-4 py-3 whitespace-nowrap">
                      <DecisionBadge decision={evt.decision} />
                    </td>
                    <td className="px-4 py-3 text-muted-foreground whitespace-nowrap font-mono text-[11px]">
                      {evt.model_version || "default"}
                    </td>
                    <td className="px-4 py-3 text-muted-foreground max-w-xs truncate font-sans text-[11px]">
                      {evt.reasons && evt.reasons.length > 0 ? evt.reasons.join(", ") : "Standard pass"}
                    </td>
                    <td className="px-4 py-3 whitespace-nowrap">
                      <span className="inline-flex items-center gap-1 text-[11px] text-emerald-400">
                        <CheckCircle2 className="h-3 w-3" /> Sealed
                      </span>
                    </td>
                  </tr>
                ))
              )}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
}
