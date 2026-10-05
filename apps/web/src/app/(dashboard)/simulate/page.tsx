"use client";

import { useState } from "react";
import { api } from "@/lib/api";
import { DecisionOutcome } from "@/types";
import { DecisionBadge, MoneyDisplay, ChannelBadge } from "@/components/shared/banking";
import {
  PlayCircle,
  Zap,
  ShieldCheck,
  ShieldAlert,
  Sliders,
  Sparkles,
  RotateCw,
  CheckCircle2,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";

function generateUuid(): string {
  if (typeof crypto !== "undefined" && crypto.randomUUID) {
    return crypto.randomUUID();
  }
  return "xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx".replace(/[xy]/g, (c) => {
    const r = (Math.random() * 16) | 0;
    const v = c === "x" ? r : (r & 0x3) | 0x8;
    return v.toString(16);
  });
}

interface PresetScenario {
  name: string;
  desc: string;
  expected: "ALLOW" | "REVIEW" | "ESCALATE" | "BLOCK";
  data: {
    user_id: string;
    merchant_id: string;
    amount: string;
    currency: string;
    channel: string;
    device_id: string;
    ip_address: string;
    merchant_category: string;
  };
}

const PRESETS: PresetScenario[] = [
  {
    name: "Routine Domestic Grocery",
    desc: "Low-risk retail POS swipe under normal limits ($48.50)",
    expected: "ALLOW",
    data: {
      user_id: "0e8224c9-1c1b-4fd1-a2d5-e092967453a5",
      merchant_id: "merch_wholefoods_98",
      amount: "48.50",
      currency: "USD",
      channel: "pos",
      device_id: "dev_terminal_01",
      ip_address: "198.51.100.12",
      merchant_category: "supermarket",
    },
  },
  {
    name: "High Value Wire Transfer",
    desc: "Single transfer exceeding $15,000 threshold ($18,500.00)",
    expected: "REVIEW",
    data: {
      user_id: "4a2b97c0-1122-4334-a556-e77889900112",
      merchant_id: "merch_escrow_chase",
      amount: "18500.00",
      currency: "USD",
      channel: "api",
      device_id: "dev_sec_token_99",
      ip_address: "203.0.113.45",
      merchant_category: "financial_services",
    },
  },
  {
    name: "Massive Offshore Escalate",
    desc: "Unusual massive capital reallocation ($145,000.00)",
    expected: "ESCALATE",
    data: {
      user_id: "5b3c08d1-2233-4445-b667-f88990011223",
      merchant_id: "merch_zurich_vault",
      amount: "145000.00",
      currency: "USD",
      channel: "web",
      device_id: "dev_macbook_pro_04",
      ip_address: "195.176.3.19",
      merchant_category: "luxury_goods",
    },
  },
  {
    name: "Suspicious ATM Card Sweep",
    desc: "High-risk velocity transaction with anomalous flags",
    expected: "BLOCK",
    data: {
      user_id: "6c4d19e2-3344-4556-c778-099001122334",
      merchant_id: "atm_kiosk_darkweb",
      amount: "800.00",
      currency: "USD",
      channel: "atm",
      device_id: "dev_skimmer_clone",
      ip_address: "185.220.101.5",
      merchant_category: "cash_advance",
    },
  },
];

export default function SimulatorPage() {
  const [formData, setFormData] = useState(PRESETS[0].data);
  const [submitting, setSubmitting] = useState(false);
  const [result, setResult] = useState<DecisionOutcome | null>(null);
  const [executionLatencyMs, setExecutionLatencyMs] = useState<number | null>(null);
  const [error, setError] = useState<string | null>(null);

  const applyPreset = (preset: PresetScenario) => {
    setFormData(preset.data);
    setResult(null);
    setError(null);
  };

  const handleSimulate = async (e: React.FormEvent) => {
    e.preventDefault();
    setSubmitting(true);
    setError(null);
    const start = performance.now();

    try {
      const parsedAmount = parseFloat(formData.amount);
      const amountMinor = Math.round(parsedAmount * 100);

      const payload = {
        transaction_id: generateUuid(),
        user_id: formData.user_id,
        event_type: "payment",
        channel: formData.channel,
        amount_minor: amountMinor,
        currency: formData.currency,
        metadata: {
          merchant_id: formData.merchant_id,
          device_id: formData.device_id,
          ip_address: formData.ip_address,
          merchant_category: formData.merchant_category,
        },
      };

      const res = await api.post<DecisionOutcome>("/transactions", payload);
      setExecutionLatencyMs(Math.round((performance.now() - start) * 100) / 100);
      setResult(res);
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : "Failed to run simulation";
      setError(msg);
    } finally {
      setSubmitting(false);
    }
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex flex-col gap-1">
        <div className="flex items-center gap-2">
          <span className="rounded-md bg-primary/10 px-2 py-0.5 text-xs font-semibold text-primary">
            ENGINE STUDIO
          </span>
          <span className="text-xs text-muted-foreground font-mono">v1.4.0-rust</span>
        </div>
        <h1 className="text-2xl font-bold tracking-tight text-foreground">
          Live Transaction Simulator
        </h1>
        <p className="text-sm text-muted-foreground">
          Inject test payloads against the real-time rule pipeline, sliding-window feature extractor, and embedded ML model.
        </p>
      </div>

      {/* Preset Scenarios */}
      <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-4">
        {PRESETS.map((p) => (
          <button
            key={p.name}
            onClick={() => applyPreset(p)}
            type="button"
            className="flex flex-col items-start gap-1 rounded-lg border border-border/70 bg-card p-3.5 text-left transition hover:border-primary/50 hover:bg-card/80"
          >
            <div className="flex w-full items-center justify-between">
              <span className="font-semibold text-xs text-foreground">{p.name}</span>
              <DecisionBadge decision={p.expected} />
            </div>
            <p className="text-[11px] text-muted-foreground leading-snug mt-1">{p.desc}</p>
            <div className="mt-2 text-[10px] font-mono text-primary flex items-center gap-1">
              <Sparkles className="h-3 w-3" />
              Load Scenario
            </div>
          </button>
        ))}
      </div>

      <div className="grid gap-6 lg:grid-cols-12">
        {/* Form Column */}
        <div className="lg:col-span-6 space-y-4">
          <div className="rounded-xl border border-border/80 bg-card p-5 shadow-sm">
            <div className="flex items-center justify-between pb-3 border-b border-border/60 mb-4">
              <div className="flex items-center gap-2 font-semibold text-sm text-foreground">
                <Sliders className="h-4 w-4 text-primary" />
                Payload Parameters
              </div>
              <span className="text-[11px] font-mono text-muted-foreground">POST /api/v1/transactions</span>
            </div>

            <form onSubmit={handleSimulate} className="space-y-4">
              <div className="grid grid-cols-2 gap-3">
                <div className="space-y-1.5">
                  <Label htmlFor="amount" className="text-xs">Transaction Amount ($)</Label>
                  <Input
                    id="amount"
                    type="number"
                    step="0.01"
                    value={formData.amount}
                    onChange={(e) => setFormData({ ...formData, amount: e.target.value })}
                    required
                    className="font-mono text-sm"
                  />
                </div>
                <div className="space-y-1.5">
                  <Label htmlFor="currency" className="text-xs">Currency</Label>
                  <Input
                    id="currency"
                    value={formData.currency}
                    onChange={(e) => setFormData({ ...formData, currency: e.target.value })}
                    required
                    className="font-mono text-sm"
                  />
                </div>
              </div>

              <div className="grid grid-cols-2 gap-3">
                <div className="space-y-1.5">
                  <Label htmlFor="channel" className="text-xs">Channel</Label>
                  <select
                    id="channel"
                    value={formData.channel}
                    onChange={(e) => setFormData({ ...formData, channel: e.target.value })}
                    className="w-full rounded-md border border-input bg-background px-3 py-2 text-sm font-mono shadow-sm focus:outline-none focus:ring-1 focus:ring-ring"
                  >
                    <option value="pos">POS (Card Present)</option>
                    <option value="web">Web (E-Commerce)</option>
                    <option value="mobile">Mobile App</option>
                    <option value="api">API Direct Gateway</option>
                    <option value="atm">ATM Withdrawal</option>
                    <option value="batch">Batch Settlement</option>
                  </select>
                </div>
                <div className="space-y-1.5">
                  <Label htmlFor="merchant_category" className="text-xs">Merchant Category</Label>
                  <Input
                    id="merchant_category"
                    value={formData.merchant_category}
                    onChange={(e) => setFormData({ ...formData, merchant_category: e.target.value })}
                    className="font-mono text-sm"
                  />
                </div>
              </div>

              <div className="space-y-1.5">
                <Label htmlFor="user_id" className="text-xs">Customer User UUID</Label>
                <Input
                  id="user_id"
                  value={formData.user_id}
                  onChange={(e) => setFormData({ ...formData, user_id: e.target.value })}
                  required
                  className="font-mono text-sm"
                />
              </div>

              <div className="space-y-1.5">
                <Label htmlFor="merchant_id" className="text-xs">Merchant ID</Label>
                <Input
                  id="merchant_id"
                  value={formData.merchant_id}
                  onChange={(e) => setFormData({ ...formData, merchant_id: e.target.value })}
                  required
                  className="font-mono text-sm"
                />
              </div>

              <div className="grid grid-cols-2 gap-3">
                <div className="space-y-1.5">
                  <Label htmlFor="ip_address" className="text-xs">IP Address</Label>
                  <Input
                    id="ip_address"
                    value={formData.ip_address}
                    onChange={(e) => setFormData({ ...formData, ip_address: e.target.value })}
                    className="font-mono text-sm"
                  />
                </div>
                <div className="space-y-1.5">
                  <Label htmlFor="device_id" className="text-xs">Device Fingerprint</Label>
                  <Input
                    id="device_id"
                    value={formData.device_id}
                    onChange={(e) => setFormData({ ...formData, device_id: e.target.value })}
                    className="font-mono text-sm"
                  />
                </div>
              </div>

              {error && (
                <div className="rounded-md border border-destructive/30 bg-destructive/10 p-3 text-xs text-red-400">
                  {error}
                </div>
              )}

              <Button
                type="submit"
                className="w-full gap-2 font-medium"
                disabled={submitting}
              >
                {submitting ? (
                  <>
                    <RotateCw className="h-4 w-4 animate-spin" />
                    Evaluating via Rust Core...
                  </>
                ) : (
                  <>
                    <PlayCircle className="h-4 w-4" />
                    Execute Transaction Decision
                  </>
                )}
              </Button>
            </form>
          </div>
        </div>

        {/* Results Column */}
        <div className="lg:col-span-6 space-y-4">
          <div className="rounded-xl border border-border/80 bg-card p-5 shadow-sm min-h-[460px] flex flex-col justify-between">
            <div>
              <div className="flex items-center justify-between pb-3 border-b border-border/60 mb-4">
                <div className="flex items-center gap-2 font-semibold text-sm text-foreground">
                  <Zap className="h-4 w-4 text-primary" />
                  Decision Inspection Output
                </div>
                {executionLatencyMs !== null && (
                  <span className="text-[11px] font-mono text-emerald-400 bg-emerald-500/10 px-2 py-0.5 rounded border border-emerald-500/20">
                    Roundtrip: {executionLatencyMs} ms
                  </span>
                )}
              </div>

              {!result ? (
                <div className="flex flex-col items-center justify-center py-16 text-center text-muted-foreground space-y-3">
                  <div className="h-12 w-12 rounded-full bg-muted/30 flex items-center justify-center border border-border">
                    <Zap className="h-6 w-6 text-muted-foreground/60" />
                  </div>
                  <p className="text-sm font-medium text-foreground">Ready for Evaluation</p>
                  <p className="text-xs max-w-sm">
                    Select a preset scenario above or enter custom parameters, then execute the decision against the pipeline.
                  </p>
                </div>
              ) : (
                <div className="space-y-4">
                  {/* Verdict Banner */}
                  <div className="flex items-center justify-between rounded-lg border border-border/80 bg-background/50 p-4">
                    <div>
                      <span className="text-[11px] uppercase tracking-wider text-muted-foreground font-mono">
                        Engine Verdict
                      </span>
                      <div className="mt-1 flex items-center gap-2">
                        <DecisionBadge decision={result.decision} />
                        <span className="text-xs text-muted-foreground font-mono">
                          Event: {result.audit_event?.event_id?.substring(0, 14)}...
                        </span>
                      </div>
                    </div>
                    <div className="text-right">
                      <span className="text-[11px] uppercase tracking-wider text-muted-foreground font-mono">
                        Transaction
                      </span>
                      <div className="mt-1 font-mono text-sm font-bold text-foreground">
                        {formData.amount} {formData.currency}
                      </div>
                    </div>
                  </div>

                  {/* Rules & Explanations */}
                  <div className="rounded-lg border border-border/80 bg-background/40 p-4 space-y-2">
                    <div className="text-xs font-semibold text-foreground flex items-center gap-1.5">
                      <ShieldAlert className="h-3.5 w-3.5 text-primary" />
                      Triggered Rules & Explanations
                    </div>
                    {result.reasons && result.reasons.length > 0 ? (
                      <ul className="space-y-1.5">
                        {result.reasons.map((r, i) => (
                          <li key={i} className="text-xs text-foreground/90 flex items-start gap-2 bg-card p-2 rounded border border-border/50">
                            <span className="text-primary font-mono font-bold">•</span>
                            <span>{r}</span>
                          </li>
                        ))}
                      </ul>
                    ) : (
                      <p className="text-xs text-muted-foreground">
                        No adverse rules fired. Transaction cleared baseline security checks.
                      </p>
                    )}
                  </div>

                  {/* Cryptographic & Model Snapshot */}
                  <div className="grid grid-cols-2 gap-3 text-xs">
                    <div className="rounded-lg border border-border/80 bg-background/40 p-3 space-y-1">
                      <span className="text-[10px] uppercase font-mono text-muted-foreground">ML Model</span>
                      <div className="font-mono text-foreground font-semibold flex items-center gap-1">
                        <CheckCircle2 className="h-3.5 w-3.5 text-emerald-400" />
                        {result.audit_event?.model_version || "fraud_xgb_v2"}
                      </div>
                      <p className="text-[10px] text-muted-foreground">Embedded ONNX / Rust engine</p>
                    </div>

                    <div className="rounded-lg border border-border/80 bg-background/40 p-3 space-y-1">
                      <span className="text-[10px] uppercase font-mono text-muted-foreground">Audit Proof</span>
                      <div className="font-mono text-foreground font-semibold flex items-center gap-1">
                        <ShieldCheck className="h-3.5 w-3.5 text-primary" />
                        SHA-256 Chained
                      </div>
                      <p className="text-[10px] text-muted-foreground">Audit ID: {result.audit_event?.audit_id?.substring(0, 10)}...</p>
                    </div>
                  </div>

                  {/* Raw JSON viewer */}
                  <div className="rounded-lg border border-border/80 bg-background/60 p-3">
                    <div className="text-[10px] uppercase font-mono text-muted-foreground mb-1">
                      Raw JSON Response
                    </div>
                    <pre className="text-[11px] font-mono text-foreground/80 overflow-x-auto max-h-36 p-2 bg-black/40 rounded">
                      {JSON.stringify(result, null, 2)}
                    </pre>
                  </div>
                </div>
              )}
            </div>

            <div className="pt-3 border-t border-border/60 text-[11px] text-muted-foreground flex items-center justify-between">
              <span>All evaluations automatically committed to recent decision buffer.</span>
              <span className="font-mono text-primary">CapitalCore Banking</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
