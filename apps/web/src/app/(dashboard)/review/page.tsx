"use client";

import { useEffect, useState } from "react";
import { api } from "@/lib/api";
import { StoredDecision } from "@/types";
import { DecisionBadge, MoneyDisplay, ChannelBadge } from "@/components/shared/banking";
import {
  AlertTriangle,
  ShieldAlert,
  CheckCircle2,
  XCircle,
  ArrowUpRight,
  RotateCw,
} from "lucide-react";
import { Button } from "@/components/ui/button";

export default function ReviewQueuePage() {
  const [items, setItems] = useState<StoredDecision[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [resolvedIds, setResolvedIds] = useState<Record<string, "APPROVED" | "BLOCKED" | "ESCALATED">>({});
  const [actionMessage, setActionMessage] = useState<string | null>(null);

  const fetchQueue = async () => {
    setIsLoading(true);
    try {
      const [reviewData, escalateData] = await Promise.all([
        api.get<StoredDecision[]>("/decisions?status=REVIEW&limit=50"),
        api.get<StoredDecision[]>("/decisions?status=ESCALATE&limit=50"),
      ]);
      setItems([...escalateData, ...reviewData]);
    } catch (err) {
      console.error("Failed to load review queue", err);
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    fetchQueue();
  }, []);

  const handleResolve = (id: string, action: "APPROVED" | "BLOCKED" | "ESCALATED") => {
    setResolvedIds((prev) => ({ ...prev, [id]: action }));
    setActionMessage(`Transaction ${id.substring(0, 14)}... marked as ${action} by Risk Analyst.`);
    setTimeout(() => setActionMessage(null), 4000);
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <div className="flex items-center gap-2">
            <span className="rounded-md bg-amber-500/10 px-2 py-0.5 text-xs font-semibold text-amber-400">
              OPERATIONS QUEUE
            </span>
            <span className="text-xs text-muted-foreground font-mono">Manual Intervention Required</span>
          </div>
          <h1 className="text-2xl font-bold tracking-tight text-foreground mt-1">
            Review & Escalation Queue
          </h1>
          <p className="text-sm text-muted-foreground">
            Triage high-value, velocity bursts, or multi-factor anomalies flagged by the rule engine.
          </p>
        </div>

        <div className="flex items-center gap-2">
          <Button
            variant="outline"
            size="sm"
            onClick={fetchQueue}
            disabled={isLoading}
            className="gap-1.5 text-xs"
          >
            <RotateCw className={`h-3.5 w-3.5 ${isLoading ? "animate-spin" : ""}`} />
            Refresh Queue
          </Button>
        </div>
      </div>

      {actionMessage && (
        <div className="p-3 rounded-lg border border-primary/30 bg-primary/10 text-xs text-primary font-mono flex items-center gap-2 animate-in fade-in">
          <CheckCircle2 className="h-4 w-4 shrink-0" />
          <span>{actionMessage}</span>
        </div>
      )}

      {/* Queue items */}
      <div className="space-y-3">
        {items.length === 0 ? (
          <div className="rounded-xl border border-border/80 bg-card p-12 text-center text-muted-foreground">
            {isLoading ? "Fetching flagged cases from engine..." : "All flagged transactions resolved! Queue is clear."}
          </div>
        ) : (
          items.map((item) => {
            const resolution = resolvedIds[item.id];

            return (
              <div
                key={item.id}
                className={`rounded-xl border p-5 transition shadow-sm bg-card ${
                  resolution
                    ? "opacity-60 border-border/40"
                    : item.decision === "ESCALATE"
                    ? "border-red-500/30 hover:border-red-500/50"
                    : "border-amber-500/30 hover:border-amber-500/50"
                }`}
              >
                <div className="flex flex-col lg:flex-row lg:items-center justify-between gap-4">
                  <div className="space-y-2">
                    <div className="flex items-center gap-3">
                      <DecisionBadge decision={item.decision} />
                      <span className="font-mono text-xs font-bold text-foreground">
                        {item.transaction_id.substring(0, 14)}...
                      </span>
                      <ChannelBadge channel={item.channel} />
                      <span className="text-[11px] font-mono text-muted-foreground">
                        {new Date(item.created_at).toLocaleTimeString()}
                      </span>
                    </div>

                    <div className="flex flex-wrap items-center gap-x-6 gap-y-1 text-xs">
                      <span className="text-muted-foreground">
                        User: <strong className="text-foreground font-mono">{item.user_id.substring(0, 14)}...</strong>
                      </span>
                      <span className="text-muted-foreground">
                        Merchant: <strong className="text-foreground font-mono">{item.metadata?.merchant_id || "N/A"}</strong>
                      </span>
                      <span className="text-muted-foreground">
                        Amount:{" "}
                        <strong className="text-foreground font-mono">
                          <MoneyDisplay amount={item.amount_minor / 100} currency={item.currency} />
                        </strong>
                      </span>
                    </div>

                    {item.reasons && item.reasons.length > 0 && (
                      <div className="rounded-md bg-background/50 border border-border/60 p-2 text-xs text-foreground/90 font-mono">
                        <span className="text-amber-400 font-semibold">Flags: </span>
                        {item.reasons.join(" • ")}
                      </div>
                    )}
                  </div>

                  {/* Actions */}
                  <div className="flex flex-wrap items-center gap-2 self-start lg:self-center">
                    {resolution ? (
                      <span className="text-xs font-mono font-bold text-muted-foreground px-3 py-1.5 rounded bg-muted/40 border border-border">
                        STATUS: {resolution}
                      </span>
                    ) : (
                      <>
                        <Button
                          size="sm"
                          variant="outline"
                          onClick={() => handleResolve(item.id, "APPROVED")}
                          className="text-xs gap-1.5 border-emerald-500/30 hover:bg-emerald-500/10 text-emerald-400"
                        >
                          <CheckCircle2 className="h-3.5 w-3.5" />
                          Approve Override
                        </Button>
                        <Button
                          size="sm"
                          variant="outline"
                          onClick={() => handleResolve(item.id, "BLOCKED")}
                          className="text-xs gap-1.5 border-red-500/30 hover:bg-red-500/10 text-red-400"
                        >
                          <XCircle className="h-3.5 w-3.5" />
                          Confirm Block
                        </Button>
                        <Button
                          size="sm"
                          onClick={() => handleResolve(item.id, "ESCALATED")}
                          className="text-xs gap-1.5 bg-primary/20 hover:bg-primary/30 text-primary border border-primary/40"
                        >
                          <ArrowUpRight className="h-3.5 w-3.5" />
                          SAR Escalation
                        </Button>
                      </>
                    )}
                  </div>
                </div>
              </div>
            );
          })
        )}
      </div>
    </div>
  );
}
