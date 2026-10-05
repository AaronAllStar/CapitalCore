"use client";

import { useEffect, useState } from "react";
import { api } from "@/lib/api";
import { ModelInfo } from "@/types";
import {
  Cpu,
  CheckCircle2,
  ShieldCheck,
  RotateCw,
  GitBranch,
} from "lucide-react";
import { Button } from "@/components/ui/button";

export default function ModelPage() {
  const [model, setModel] = useState<ModelInfo | null>(null);
  const [loading, setLoading] = useState(true);

  const fetchModel = async () => {
    setLoading(true);
    try {
      const data = await api.get<ModelInfo>("/model");
      setModel(data);
    } catch (err) {
      console.error("Failed to load model details", err);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchModel();
  }, []);

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <div className="flex items-center gap-2">
            <span className="rounded-md bg-purple-500/10 px-2 py-0.5 text-xs font-semibold text-purple-400">
              MACHINE LEARNING SPEC
            </span>
            <span className="text-xs text-muted-foreground font-mono">Embedded Inference</span>
          </div>
          <h1 className="text-2xl font-bold tracking-tight text-foreground mt-1">
            Fraud Model Card & Feature Catalog
          </h1>
          <p className="text-sm text-muted-foreground">
            Embedded ONNX / Rust decision engine combining tree-based ensembles with sliding-window aggregate features.
          </p>
        </div>

        <Button
          variant="outline"
          size="sm"
          onClick={fetchModel}
          disabled={loading}
          className="gap-1.5 text-xs"
        >
          <RotateCw className={`h-3.5 w-3.5 ${loading ? "animate-spin" : ""}`} />
          Reload Card
        </Button>
      </div>

      {/* Model Spec Grid */}
      <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
        <div className="rounded-xl border border-border/80 bg-card p-4 space-y-1">
          <span className="text-[11px] font-mono uppercase text-muted-foreground">Accuracy</span>
          <div className="text-2xl font-bold text-emerald-400 font-mono">
            {model?.accuracy || "99.60%"}
          </div>
          <p className="text-[10px] text-muted-foreground">Evaluated on cross-validation holdout</p>
        </div>

        <div className="rounded-xl border border-border/80 bg-card p-4 space-y-1">
          <span className="text-[11px] font-mono uppercase text-muted-foreground">Precision</span>
          <div className="text-2xl font-bold text-foreground font-mono">
            {model?.precision || "96.55%"}
          </div>
          <p className="text-[10px] text-muted-foreground">Low false-positive rate on high volume</p>
        </div>

        <div className="rounded-xl border border-border/80 bg-card p-4 space-y-1">
          <span className="text-[11px] font-mono uppercase text-muted-foreground">Recall (Fraud Catch)</span>
          <div className="text-2xl font-bold text-primary font-mono">
            {model?.recall || "100.00%"}
          </div>
          <p className="text-[10px] text-muted-foreground">Minimizes unauthorized chargebacks</p>
        </div>

        <div className="rounded-xl border border-border/80 bg-card p-4 space-y-1">
          <span className="text-[11px] font-mono uppercase text-muted-foreground">Inference Target</span>
          <div className="text-2xl font-bold text-foreground font-mono">
            {model?.latency_p99_us ? `${model.latency_p99_us} μs` : "< 15 μs"}
          </div>
          <p className="text-[10px] text-muted-foreground">Embedded Rust memory model, no network hops</p>
        </div>
      </div>

      {/* Feature Catalog & Pipeline Overview */}
      <div className="grid gap-6 lg:grid-cols-12">
        <div className="lg:col-span-7 space-y-4">
          <div className="rounded-xl border border-border/80 bg-card p-5 shadow-sm space-y-4">
            <div className="flex items-center justify-between pb-3 border-b border-border/60">
              <div className="flex items-center gap-2 font-semibold text-sm text-foreground">
                <Cpu className="h-4 w-4 text-purple-400" />
                Feature Extraction Catalog (Sliding Window)
              </div>
              <span className="text-xs font-mono text-muted-foreground">
                {model?.features_catalog?.length || 9} Active Features
              </span>
            </div>

            <div className="space-y-2">
              {(
                model?.features_catalog || [
                  { name: "amount_minor", type: "int64", window: "instant", description: "Transaction amount in minor units" },
                  { name: "user_txn_count_5m", type: "int64", window: "5 minutes", description: "Trailing transaction frequency" },
                  { name: "user_txn_sum_5m", type: "int64", window: "5 minutes", description: "Trailing aggregate volume" },
                  { name: "user_txn_count_1h", type: "int64", window: "1 hour", description: "Trailing transaction count" },
                  { name: "user_txn_sum_1h", type: "int64", window: "1 hour", description: "Trailing aggregate volume" },
                  { name: "user_txn_max_1h", type: "int64", window: "1 hour", description: "Peak single transaction amount" },
                  { name: "user_txn_count_24h", type: "int64", window: "24 hours", description: "Daily transaction count" },
                  { name: "user_txn_sum_24h", type: "int64", window: "24 hours", description: "Daily cumulative volume" },
                  { name: "user_distinct_merchants_24h", type: "int64", window: "24 hours", description: "Distinct merchants visited" }
                ]
              ).map((feat, idx) => (
                <div
                  key={idx}
                  className="flex items-center justify-between rounded-lg border border-border/60 bg-background/40 p-3 text-xs"
                >
                  <div className="space-y-0.5">
                    <div className="flex items-center gap-2 font-mono">
                      <span className="text-muted-foreground font-semibold">#{idx + 1}</span>
                      <span className="text-foreground font-semibold">{feat.name}</span>
                      <span className="text-[10px] text-muted-foreground">({feat.type})</span>
                    </div>
                    <p className="text-[11px] text-muted-foreground font-sans">{feat.description}</p>
                  </div>
                  <span className="rounded bg-primary/10 px-2 py-0.5 text-[10px] font-mono text-primary whitespace-nowrap">
                    {feat.window}
                  </span>
                </div>
              ))}
            </div>
          </div>
        </div>

        {/* Deployment Metadata */}
        <div className="lg:col-span-5 space-y-4">
          <div className="rounded-xl border border-border/80 bg-card p-5 shadow-sm space-y-4">
            <div className="flex items-center gap-2 font-semibold text-sm text-foreground pb-3 border-b border-border/60">
              <GitBranch className="h-4 w-4 text-primary" />
              Model Governance & Versioning
            </div>

            <div className="space-y-3 text-xs">
              <div className="flex justify-between py-1.5 border-b border-border/40">
                <span className="text-muted-foreground">Model Architecture</span>
                <span className="font-mono font-bold text-foreground">
                  {model?.model_name || "CapitalCore Gradient Decision Tree"}
                </span>
              </div>
              <div className="flex justify-between py-1.5 border-b border-border/40">
                <span className="text-muted-foreground">Active Model Version</span>
                <span className="font-mono font-bold text-foreground">
                  {model?.version || "v1.2.0-onnx"}
                </span>
              </div>
              <div className="flex justify-between py-1.5 border-b border-border/40">
                <span className="text-muted-foreground">Runtime Engine</span>
                <span className="font-mono text-foreground">{model?.runtime || "Embedded Rust Native (edge-ml)"}</span>
              </div>
              <div className="flex justify-between py-1.5 border-b border-border/40">
                <span className="text-muted-foreground">Arbitration Policy</span>
                <span className="font-mono text-emerald-400 font-semibold text-[11px]">
                  {model?.arbitration_strategy || "MostRestrictive (Block > Escalate > Review > Allow)"}
                </span>
              </div>
              <div className="flex justify-between py-1.5 border-b border-border/40">
                <span className="text-muted-foreground">Fraud Cutoff Threshold</span>
                <span className="font-mono text-foreground">
                  {model?.thresholds?.fraud_cutoff_bps ? `${model.thresholds.fraud_cutoff_bps} bps` : "5000 bps (50%)"}
                </span>
              </div>
              <div className="flex justify-between py-1.5 border-b border-border/40">
                <span className="text-muted-foreground">Review Cutoff Threshold</span>
                <span className="font-mono text-foreground">
                  {model?.thresholds?.review_cutoff_bps ? `${model.thresholds.review_cutoff_bps} bps` : "2500 bps (25%)"}
                </span>
              </div>
            </div>

            <div className="rounded-lg border border-primary/20 bg-primary/5 p-3 text-xs text-muted-foreground space-y-1">
              <div className="font-semibold text-foreground flex items-center gap-1.5">
                <ShieldCheck className="h-3.5 w-3.5 text-primary" />
                Deterministic Explainability
              </div>
              <p>
                Every inference maps directly to an explainable feature attribution vector included in the immutable transaction audit entry.
              </p>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
