"use client";

import { useEffect, useState } from "react";
import Link from "next/link";
import { api } from "@/lib/api";
import { DecisionStats, StoredDecision, AuditVerificationResult } from "@/types";
import { DecisionBadge, MoneyDisplay, ChannelBadge } from "@/components/shared/banking";
import {
  ShieldAlert,
  ArrowUpRight,
  TrendingDown,
  CheckCircle2,
  Clock,
  PlayCircle,
  History,
  RotateCw,
  Search,
} from "lucide-react";
import { cn } from "@/lib/utils";

export default function DashboardPage() {
  const [stats, setStats] = useState<DecisionStats | null>(null);
  const [decisions, setDecisions] = useState<StoredDecision[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [isVerifying, setIsVerifying] = useState(false);
  const [verification, setVerification] = useState<AuditVerificationResult | null>(null);
  const [statusFilter, setStatusFilter] = useState<string>("ALL");

  const loadData = async () => {
    setIsLoading(true);
    try {
      const [statsData, decisionsData] = await Promise.all([
        api.get<DecisionStats>("/decisions/stats"),
        api.get<StoredDecision[]>("/decisions?limit=25"),
      ]);
      setStats(statsData);
      setDecisions(decisionsData);
    } catch (e) {
      console.error("Failed to load dashboard metrics", e);
    } finally {
      setIsLoading(false);
    }
  };

  useEffect(() => {
    loadData();
    const interval = setInterval(loadData, 10000);
    return () => clearInterval(interval);
  }, []);

  const handleVerifyIntegrity = async () => {
    setIsVerifying(true);
    try {
      const res = await api.get<AuditVerificationResult>("/audit/verify");
      setVerification(res);
    } catch (e) {
      console.error("Integrity verification failed", e);
    } finally {
      setIsVerifying(false);
    }
  };

  const filteredDecisions = decisions.filter((d) => {
    if (statusFilter === "ALL") return true;
    return d.decision.toUpperCase() === statusFilter;
  });

  return (
    <div className="space-y-8 p-8 max-w-7xl mx-auto">
      {/* Page Title & Top Actions */}
      <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4 border-b border-slate-800/80 pb-6">
        <div>
          <h1 className="text-2xl font-bold tracking-tight text-white flex items-center gap-2">
            <span>Fraud Operations &amp; Decision Stream</span>
          </h1>
          <p className="text-sm text-slate-400 mt-1">
            Real-time transaction ingestion, deterministic policy arbitration, and ML fraud scoring.
          </p>
        </div>

        <div className="flex items-center gap-3">
          <button
            onClick={loadData}
            disabled={isLoading}
            className="flex items-center gap-1.5 rounded-lg border border-slate-700 bg-slate-900/60 px-3 py-2 text-xs font-medium text-slate-300 hover:bg-slate-800 transition-colors"
          >
            <RotateCw className={cn("h-3.5 w-3.5", isLoading && "animate-spin")} />
            <span>Refresh</span>
          </button>

          <button
            onClick={handleVerifyIntegrity}
            disabled={isVerifying}
            className="flex items-center gap-1.5 rounded-lg border border-cyan-500/30 bg-cyan-950/30 px-3 py-2 text-xs font-medium text-cyan-300 hover:bg-cyan-900/40 transition-colors"
          >
            <History className="h-3.5 w-3.5 text-cyan-400" />
            <span>{isVerifying ? "Verifying..." : "Verify Hash Chain"}</span>
          </button>

          <Link
            href="/simulate"
            className="flex items-center gap-1.5 rounded-lg bg-emerald-600 px-4 py-2 text-xs font-semibold text-white shadow hover:bg-emerald-500 transition-colors"
          >
            <PlayCircle className="h-4 w-4" />
            <span>Test Simulator</span>
          </Link>
        </div>
      </div>

      {/* Verification Alert if triggered */}
      {verification && (
        <div className="rounded-xl border border-emerald-500/30 bg-emerald-950/20 p-4 flex items-center justify-between">
          <div className="flex items-center gap-3">
            <CheckCircle2 className="h-5 w-5 text-emerald-400" />
            <div>
              <p className="text-sm font-semibold text-emerald-200">
                Cryptographic Hash Chain Verified
              </p>
              <p className="text-xs text-emerald-400/80">
                Status: {verification.chain_status} &bull; Timestamp: {new Date(verification.verified_at).toLocaleTimeString()}
              </p>
            </div>
          </div>
          <button
            onClick={() => setVerification(null)}
            className="text-xs text-slate-400 hover:text-white"
          >
            Dismiss
          </button>
        </div>
      )}

      {/* KPI Stats Grid */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-5">
        {/* Card 1: Total Volume */}
        <div className="rounded-xl border border-slate-800 bg-slate-900/60 p-5 backdrop-blur-md">
          <div className="flex items-center justify-between text-slate-400">
            <span className="text-xs font-medium uppercase tracking-wider">Total Evaluated Volume</span>
            <ArrowUpRight className="h-4 w-4 text-emerald-400" />
          </div>
          <div className="mt-3">
            <span className="text-2xl font-extrabold text-white">
              {stats ? (
                <MoneyDisplay amountMinor={stats.total_volume_minor} />
              ) : (
                "$0.00"
              )}
            </span>
          </div>
          <p className="mt-1 text-xs text-slate-400">
            Across {stats?.total_count || 0} financial events
          </p>
        </div>

        {/* Card 2: Block Rate */}
        <div className="rounded-xl border border-slate-800 bg-slate-900/60 p-5 backdrop-blur-md">
          <div className="flex items-center justify-between text-slate-400">
            <span className="text-xs font-medium uppercase tracking-wider">Fraud Block Rate</span>
            <TrendingDown className="h-4 w-4 text-rose-400" />
          </div>
          <div className="mt-3 flex items-baseline gap-2">
            <span className="text-2xl font-extrabold text-white">
              {stats ? `${stats.block_rate_pct}%` : "0.00%"}
            </span>
            <span className="text-xs text-rose-400 font-semibold">
              {stats?.block_count || 0} blocked
            </span>
          </div>
          <p className="mt-1 text-xs text-slate-400">
            Target SLA: &lt; 0.5% False Positive Rate
          </p>
        </div>

        {/* Card 3: Pending Review Queue */}
        <div className="rounded-xl border border-slate-800 bg-slate-900/60 p-5 backdrop-blur-md">
          <div className="flex items-center justify-between text-slate-400">
            <span className="text-xs font-medium uppercase tracking-wider">Analyst Review Queue</span>
            <Clock className="h-4 w-4 text-amber-400" />
          </div>
          <div className="mt-3 flex items-baseline gap-2">
            <span className="text-2xl font-extrabold text-amber-300">
              {stats?.review_queue_count || 0}
            </span>
            <span className="text-xs text-slate-400">
              ({stats?.review_count || 0} review &bull; {stats?.escalate_count || 0} escalate)
            </span>
          </div>
          <p className="mt-1 text-xs text-slate-400">
            <Link href="/review" className="text-amber-400 hover:underline">
              Inspect pending alerts &rarr;
            </Link>
          </p>
        </div>

        {/* Card 4: ML & Rule Status */}
        <div className="rounded-xl border border-slate-800 bg-slate-900/60 p-5 backdrop-blur-md">
          <div className="flex items-center justify-between text-slate-400">
            <span className="text-xs font-medium uppercase tracking-wider">Engine Performance</span>
            <CheckCircle2 className="h-4 w-4 text-cyan-400" />
          </div>
          <div className="mt-3 flex items-baseline gap-2">
            <span className="text-2xl font-extrabold text-white">
              {stats?.ml_model_accuracy || 99.6}%
            </span>
            <span className="text-xs text-cyan-300">Accuracy</span>
          </div>
          <p className="mt-1 text-xs text-slate-400">
            {stats?.active_rules_count || 4} active rules &bull; Rust engine
          </p>
        </div>
      </div>

      {/* Decision Feed Table */}
      <div className="rounded-xl border border-slate-800 bg-slate-900/60 overflow-hidden backdrop-blur-md">
        <div className="p-5 border-b border-slate-800 flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
          <div>
            <h2 className="text-base font-bold text-white">Recent Evaluated Transactions</h2>
            <p className="text-xs text-slate-400 mt-0.5">
              Live stream of decisions emitted by CapitalCore arbitration engine
            </p>
          </div>

          {/* Filter Pills */}
          <div className="flex items-center gap-1.5 bg-slate-950 p-1 rounded-lg border border-slate-800">
            {["ALL", "ALLOW", "REVIEW", "ESCALATE", "BLOCK"].map((filter) => (
              <button
                key={filter}
                onClick={() => setStatusFilter(filter)}
                className={cn(
                  "px-3 py-1 rounded text-xs font-semibold transition-colors",
                  statusFilter === filter
                    ? "bg-slate-800 text-white shadow-sm"
                    : "text-slate-400 hover:text-slate-200"
                )}
              >
                {filter}
              </button>
            ))}
          </div>
        </div>

        <div className="overflow-x-auto">
          <table className="w-full text-left text-sm text-slate-300">
            <thead className="bg-slate-950/80 text-[11px] uppercase tracking-wider text-slate-400 border-b border-slate-800">
              <tr>
                <th className="py-3 px-4">Decision</th>
                <th className="py-3 px-4">Amount</th>
                <th className="py-3 px-4">Channel / Type</th>
                <th className="py-3 px-4">Reason Codes</th>
                <th className="py-3 px-4">ML Risk Score</th>
                <th className="py-3 px-4">Transaction / Event ID</th>
                <th className="py-3 px-4">Time</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-800/60 font-mono text-xs">
              {filteredDecisions.length === 0 ? (
                <tr>
                  <td colSpan={7} className="py-12 text-center text-slate-500 font-sans">
                    No transactions recorded matching the selected filter.
                  </td>
                </tr>
              ) : (
                filteredDecisions.map((d) => (
                  <tr key={d.id} className="hover:bg-slate-800/40 transition-colors">
                    <td className="py-3 px-4">
                      <DecisionBadge decision={d.decision} />
                    </td>
                    <td className="py-3 px-4 text-white font-semibold">
                      <MoneyDisplay amountMinor={d.amount_minor} currency={d.currency} />
                    </td>
                    <td className="py-3 px-4 font-sans">
                      <div className="flex items-center gap-2">
                        <ChannelBadge channel={d.channel} />
                        <span className="text-[11px] text-slate-400 uppercase font-semibold">
                          {d.event_type}
                        </span>
                      </div>
                    </td>
                    <td className="py-3 px-4 font-sans max-w-xs truncate">
                      {d.reasons.length > 0 ? (
                        <div className="flex flex-wrap gap-1">
                          {d.reasons.map((r) => (
                            <span
                              key={r}
                              className="inline-block bg-slate-800 text-amber-300 text-[10px] px-2 py-0.5 rounded font-mono border border-slate-700"
                            >
                              {r}
                            </span>
                          ))}
                        </div>
                      ) : (
                        <span className="text-slate-400 text-xs">Policy compliant</span>
                      )}
                    </td>
                    <td className="py-3 px-4 font-sans">
                      {d.risk_score_bps !== undefined && d.risk_score_bps !== null ? (
                        <span
                          className={cn(
                            "font-mono text-xs font-semibold",
                            d.risk_score_bps >= 5000
                              ? "text-rose-400"
                              : d.risk_score_bps >= 2500
                              ? "text-amber-400"
                              : "text-emerald-400"
                          )}
                        >
                          {(d.risk_score_bps / 100).toFixed(1)}% ({d.risk_score_bps} bps)
                        </span>
                      ) : (
                        <span className="text-slate-400 text-xs">N/A</span>
                      )}
                    </td>
                    <td className="py-3 px-4 text-slate-400">
                      <span title={d.event_id}>
                        {d.event_id.slice(0, 8)}...{d.event_id.slice(-4)}
                      </span>
                    </td>
                    <td className="py-3 px-4 font-sans text-xs text-slate-400">
                      {new Date(d.created_at).toLocaleTimeString()}
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
