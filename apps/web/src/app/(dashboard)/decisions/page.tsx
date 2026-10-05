"use client";

import { useEffect, useState } from "react";
import { api } from "@/lib/api";
import { StoredDecision } from "@/types";
import { DecisionBadge, MoneyDisplay, ChannelBadge } from "@/components/shared/banking";
import {
  Search,
  RotateCw,
  Filter,
  CheckCircle2,
  AlertTriangle,
  ShieldAlert,
  ChevronRight,
  X,
  FileCode,
  ShieldCheck,
} from "lucide-react";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";

export default function DecisionsPage() {
  const [decisions, setDecisions] = useState<StoredDecision[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [statusFilter, setStatusFilter] = useState<string>("ALL");
  const [searchTerm, setSearchTerm] = useState("");
  const [selectedDecision, setSelectedDecision] = useState<StoredDecision | null>(null);

  const fetchDecisions = async () => {
    setIsLoading(true);
    try {
      const q = statusFilter !== "ALL" ? `?status=${statusFilter}&limit=100` : "?limit=100";
      const data = await api.get<StoredDecision[]>(`/decisions${q}`);
      setDecisions(data);
    } catch (err) {
      console.error("Failed to load decisions", err);
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    fetchDecisions();
  }, [statusFilter]);

  const filteredDecisions = decisions.filter((d) => {
    if (!searchTerm) return true;
    const term = searchTerm.toLowerCase();
    const merchant = d.metadata?.merchant_id?.toLowerCase() || "";
    return (
      d.transaction_id.toLowerCase().includes(term) ||
      d.user_id.toLowerCase().includes(term) ||
      d.event_id.toLowerCase().includes(term) ||
      merchant.includes(term)
    );
  });

  return (
    <div className="space-y-6">
      {/* Page Header */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <div className="flex items-center gap-2">
            <span className="rounded-md bg-primary/10 px-2 py-0.5 text-xs font-semibold text-primary">
              TRANSACTION EXPLORER
            </span>
            <span className="text-xs text-muted-foreground font-mono">Real-Time Ingestion</span>
          </div>
          <h1 className="text-2xl font-bold tracking-tight text-foreground mt-1">
            Decisions & Audit Records
          </h1>
          <p className="text-sm text-muted-foreground">
            Search, filter, and inspect deterministic verdicts generated across all financial channels.
          </p>
        </div>

        <div className="flex items-center gap-2">
          <Button
            variant="outline"
            size="sm"
            onClick={fetchDecisions}
            disabled={isLoading}
            className="gap-1.5 text-xs"
          >
            <RotateCw className={`h-3.5 w-3.5 ${isLoading ? "animate-spin" : ""}`} />
            Refresh Feed
          </Button>
        </div>
      </div>

      {/* Filter and Search Bar */}
      <div className="flex flex-col sm:flex-row items-stretch sm:items-center gap-3">
        <div className="relative flex-1">
          <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
          <Input
            placeholder="Filter by Transaction ID, User ID, Merchant..."
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
            className="pl-9 text-xs font-mono bg-card"
          />
        </div>

        <div className="flex items-center gap-1.5 overflow-x-auto pb-1 sm:pb-0">
          {["ALL", "ALLOW", "REVIEW", "ESCALATE", "BLOCK"].map((status) => (
            <button
              key={status}
              onClick={() => setStatusFilter(status)}
              className={`rounded-md px-3 py-1.5 text-xs font-medium transition ${
                statusFilter === status
                  ? "bg-primary text-primary-foreground shadow"
                  : "bg-card text-muted-foreground hover:bg-muted/50 border border-border/60"
              }`}
            >
              {status}
            </button>
          ))}
        </div>
      </div>

      {/* Main Decisions Table */}
      <div className="rounded-xl border border-border/80 bg-card overflow-hidden shadow-sm">
        <div className="overflow-x-auto">
          <table className="w-full text-left text-xs">
            <thead className="bg-muted/40 font-mono text-muted-foreground uppercase border-b border-border/80">
              <tr>
                <th className="px-4 py-3">Timestamp</th>
                <th className="px-4 py-3">Transaction ID</th>
                <th className="px-4 py-3">User & Merchant</th>
                <th className="px-4 py-3">Amount</th>
                <th className="px-4 py-3">Channel</th>
                <th className="px-4 py-3">Decision</th>
                <th className="px-4 py-3">Primary Trigger</th>
                <th className="px-4 py-3 text-right">Details</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-border/60 font-mono">
              {filteredDecisions.length === 0 ? (
                <tr>
                  <td colSpan={8} className="px-4 py-12 text-center text-muted-foreground">
                    {isLoading ? "Loading decision records from Rust core..." : "No decisions matching your search criteria."}
                  </td>
                </tr>
              ) : (
                filteredDecisions.map((d) => (
                  <tr
                    key={d.id}
                    onClick={() => setSelectedDecision(d)}
                    className="hover:bg-muted/30 cursor-pointer transition"
                  >
                    <td className="px-4 py-3 whitespace-nowrap text-muted-foreground">
                      {new Date(d.created_at).toLocaleTimeString()}
                    </td>
                    <td className="px-4 py-3 font-semibold text-foreground whitespace-nowrap">
                      {d.transaction_id.substring(0, 14)}...
                    </td>
                    <td className="px-4 py-3 whitespace-nowrap">
                      <div className="text-foreground">{d.user_id.substring(0, 14)}...</div>
                      <div className="text-[10px] text-muted-foreground">{d.metadata?.merchant_id || "N/A"}</div>
                    </td>
                    <td className="px-4 py-3 whitespace-nowrap font-semibold">
                      <MoneyDisplay amount={d.amount_minor / 100} currency={d.currency} />
                    </td>
                    <td className="px-4 py-3 whitespace-nowrap">
                      <ChannelBadge channel={d.channel} />
                    </td>
                    <td className="px-4 py-3 whitespace-nowrap">
                      <DecisionBadge decision={d.decision} />
                    </td>
                    <td className="px-4 py-3 text-muted-foreground max-w-xs truncate font-sans text-[11px]">
                      {d.reasons && d.reasons.length > 0 ? d.reasons[0] : "Standard Baseline Check"}
                    </td>
                    <td className="px-4 py-3 text-right whitespace-nowrap">
                      <ChevronRight className="h-4 w-4 inline text-muted-foreground" />
                    </td>
                  </tr>
                ))
              )}
            </tbody>
          </table>
        </div>
      </div>

      {/* Decision Detail Drawer / Modal */}
      {selectedDecision && (
        <div className="fixed inset-0 z-50 flex items-center justify-end bg-black/60 backdrop-blur-sm p-2 sm:p-6 animate-in fade-in duration-200">
          <div className="relative w-full max-w-xl h-full max-h-[92vh] rounded-xl border border-border bg-card shadow-2xl flex flex-col overflow-hidden">
            {/* Drawer Header */}
            <div className="flex items-center justify-between p-5 border-b border-border">
              <div>
                <span className="text-[10px] font-mono text-muted-foreground uppercase">
                  Decision Inspection
                </span>
                <h3 className="text-lg font-bold text-foreground">
                  {selectedDecision.transaction_id}
                </h3>
              </div>
              <button
                onClick={() => setSelectedDecision(null)}
                className="rounded-lg p-1.5 hover:bg-muted text-muted-foreground hover:text-foreground"
              >
                <X className="h-5 w-5" />
              </button>
            </div>

            {/* Drawer Body */}
            <div className="flex-1 overflow-y-auto p-5 space-y-5 text-xs font-sans">
              <div className="flex items-center justify-between p-4 rounded-lg bg-background/50 border border-border/80">
                <div>
                  <span className="text-[10px] font-mono text-muted-foreground uppercase">Final Verdict</span>
                  <div className="mt-1">
                    <DecisionBadge decision={selectedDecision.decision} />
                  </div>
                </div>
                <div className="text-right">
                  <span className="text-[10px] font-mono text-muted-foreground uppercase">Amount</span>
                  <div className="mt-1 font-mono text-base font-bold text-foreground">
                    <MoneyDisplay amount={selectedDecision.amount_minor / 100} currency={selectedDecision.currency} />
                  </div>
                </div>
              </div>

              {/* Core attributes */}
              <div className="grid grid-cols-2 gap-3 font-mono">
                <div className="p-3 rounded-lg border border-border/60 bg-background/30 space-y-1">
                  <span className="text-[10px] text-muted-foreground uppercase">User UUID</span>
                  <div className="font-semibold text-foreground truncate">{selectedDecision.user_id}</div>
                </div>
                <div className="p-3 rounded-lg border border-border/60 bg-background/30 space-y-1">
                  <span className="text-[10px] text-muted-foreground uppercase">Merchant</span>
                  <div className="font-semibold text-foreground">{selectedDecision.metadata?.merchant_id || "N/A"}</div>
                </div>
                <div className="p-3 rounded-lg border border-border/60 bg-background/30 space-y-1">
                  <span className="text-[10px] text-muted-foreground uppercase">Channel</span>
                  <div className="mt-0.5"><ChannelBadge channel={selectedDecision.channel} /></div>
                </div>
                <div className="p-3 rounded-lg border border-border/60 bg-background/30 space-y-1">
                  <span className="text-[10px] text-muted-foreground uppercase">Timestamp</span>
                  <div className="text-foreground">{new Date(selectedDecision.created_at).toLocaleString()}</div>
                </div>
              </div>

              {/* Reasons */}
              <div className="space-y-2">
                <span className="text-[11px] font-semibold text-foreground uppercase tracking-wider font-mono">
                  Triggered Explanations
                </span>
                {selectedDecision.reasons && selectedDecision.reasons.length > 0 ? (
                  <div className="space-y-1.5">
                    {selectedDecision.reasons.map((r, i) => (
                      <div key={i} className="flex items-start gap-2 p-2.5 rounded-lg border border-border/60 bg-background/30">
                        <ShieldAlert className="h-4 w-4 text-amber-400 shrink-0 mt-0.5" />
                        <span className="text-foreground/90">{r}</span>
                      </div>
                    ))}
                  </div>
                ) : (
                  <p className="text-muted-foreground italic">No adverse rules triggered.</p>
                )}
              </div>

              {/* Audit Proof */}
              <div className="p-4 rounded-lg border border-primary/20 bg-primary/5 space-y-2 font-mono">
                <div className="flex items-center gap-2 text-primary font-semibold text-xs">
                  <ShieldCheck className="h-4 w-4" />
                  Tamper-Evident Audit Record
                </div>
                <div className="text-[11px] text-muted-foreground">
                  Event ID: <span className="text-foreground">{selectedDecision.event_id}</span>
                </div>
                <div className="text-[11px] text-muted-foreground">
                  Audit ID: <span className="text-foreground">{selectedDecision.audit_id}</span>
                </div>
              </div>
            </div>

            {/* Drawer Footer */}
            <div className="p-4 border-t border-border flex justify-end">
              <Button size="sm" variant="outline" onClick={() => setSelectedDecision(null)}>
                Close
              </Button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
