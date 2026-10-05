import Link from "next/link";
import {
  ShieldCheck,
  Zap,
  Lock,
  ArrowRight,
  Landmark,
  Cpu,
  History,
  Activity,
  CheckCircle2,
} from "lucide-react";

export default function HomePage() {
  return (
    <div className="min-h-screen bg-slate-950 text-slate-100 flex flex-col selection:bg-emerald-500 selection:text-white">
      {/* Top Navigation */}
      <header className="border-b border-slate-800/80 bg-slate-950/80 backdrop-blur-md sticky top-0 z-50">
        <div className="mx-auto flex h-16 max-w-7xl items-center justify-between px-6">
          <div className="flex items-center gap-3">
            <div className="flex h-9 w-9 items-center justify-center rounded-lg bg-emerald-500/20 text-emerald-400 border border-emerald-500/30">
              <Landmark className="h-5 w-5" />
            </div>
            <div className="flex items-center gap-2">
              <span className="text-lg font-bold tracking-tight text-white">CapitalCore</span>
              <span className="rounded bg-emerald-500/20 px-2 py-0.5 text-[10px] font-semibold text-emerald-300 border border-emerald-500/30">
                FINANCIAL DECISION ENGINE
              </span>
            </div>
          </div>

          <div className="flex items-center gap-4">
            <Link
              href="/simulate"
              className="text-xs font-medium text-slate-300 hover:text-white transition-colors"
            >
              Simulator
            </Link>
            <Link
              href="/dashboard"
              className="inline-flex items-center gap-1.5 rounded-lg bg-emerald-600 px-4 py-2 text-xs font-semibold text-white shadow-sm hover:bg-emerald-500 transition-colors"
            >
              <span>Analyst Console</span>
              <ArrowRight className="h-3.5 w-3.5" />
            </Link>
          </div>
        </div>
      </header>

      {/* Hero Section */}
      <main className="flex-1">
        <div className="relative overflow-hidden py-24 sm:py-32">
          {/* Subtle background glow */}
          <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[600px] h-[350px] bg-emerald-500/10 blur-[120px] rounded-full pointer-events-none" />

          <div className="mx-auto max-w-5xl px-6 text-center relative z-10">
            <div className="inline-flex items-center gap-2 rounded-full border border-emerald-500/30 bg-emerald-500/10 px-3.5 py-1 text-xs font-medium text-emerald-300 mb-8">
              <ShieldCheck className="h-3.5 w-3.5 text-emerald-400" />
              <span>Next-Generation Banking Decision Infrastructure</span>
            </div>

            <h1 className="text-4xl sm:text-6xl font-extrabold tracking-tight text-white leading-tight">
              Real-Time Transaction Decisioning &amp;{" "}
              <span className="bg-gradient-to-r from-emerald-400 via-teal-300 to-cyan-400 bg-clip-text text-transparent">
                Fraud Prevention
              </span>
            </h1>

            <p className="mt-6 text-lg text-slate-300 max-w-3xl mx-auto leading-relaxed">
              CapitalCore combines deterministic compliance rule arbitration with embedded native Rust machine learning inference. Evaluates high-velocity payments, transfers, and withdrawals under sub-millisecond SLAs with tamper-evident audit logging.
            </p>

            <div className="mt-10 flex flex-wrap items-center justify-center gap-4">
              <Link
                href="/dashboard"
                className="inline-flex items-center gap-2 rounded-lg bg-emerald-500 px-6 py-3 text-sm font-bold text-slate-950 shadow-lg shadow-emerald-500/20 hover:bg-emerald-400 transition-all hover:scale-[1.02]"
              >
                <span>Launch Operations Console</span>
                <ArrowRight className="h-4 w-4" />
              </Link>
              <Link
                href="/simulate"
                className="inline-flex items-center gap-2 rounded-lg border border-slate-700 bg-slate-900/60 px-6 py-3 text-sm font-semibold text-white hover:bg-slate-800 transition-all"
              >
                <span>Test Transaction Simulator</span>
              </Link>
            </div>

            {/* Quick Metrics Bar */}
            <div className="mt-16 grid grid-cols-2 md:grid-cols-4 gap-4 border border-slate-800/80 bg-slate-900/40 rounded-2xl p-6 backdrop-blur-md">
              <div className="text-left">
                <p className="text-xs font-medium text-slate-400">ML Model Accuracy</p>
                <p className="text-2xl font-extrabold text-white mt-1">99.60%</p>
                <p className="text-[11px] text-emerald-400 mt-0.5 flex items-center gap-1">
                  <CheckCircle2 className="h-3 w-3" /> Zero-Drift Parity
                </p>
              </div>
              <div className="text-left">
                <p className="text-xs font-medium text-slate-400">P99 Decision Latency</p>
                <p className="text-2xl font-extrabold text-white mt-1">&lt; 1.0 ms</p>
                <p className="text-[11px] text-emerald-400 mt-0.5">Native Rust Execution</p>
              </div>
              <div className="text-left">
                <p className="text-xs font-medium text-slate-400">Audit Proofs</p>
                <p className="text-2xl font-extrabold text-white mt-1">100%</p>
                <p className="text-[11px] text-cyan-400 mt-0.5">Tamper-Evident Hash Chain</p>
              </div>
              <div className="text-left">
                <p className="text-xs font-medium text-slate-400">Arbitration Strategy</p>
                <p className="text-xl font-bold text-white mt-1">MostRestrictive</p>
                <p className="text-[11px] text-slate-400 mt-0.5">Block &gt; Escalate &gt; Review &gt; Allow</p>
              </div>
            </div>
          </div>
        </div>

        {/* Core Capabilities */}
        <div className="border-t border-slate-800/80 bg-slate-900/20 py-20">
          <div className="mx-auto max-w-6xl px-6">
            <h2 className="text-center text-xs font-bold uppercase tracking-wider text-emerald-400">
              Enterprise Banking Architecture
            </h2>
            <p className="mt-2 text-center text-2xl sm:text-3xl font-extrabold text-white">
              Deterministic Rules Engineered for Modern Financial Institutions
            </p>

            <div className="mt-12 grid grid-cols-1 md:grid-cols-3 gap-6">
              <div className="rounded-xl border border-slate-800 bg-slate-900/50 p-6">
                <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 mb-4">
                  <Zap className="h-5 w-5" />
                </div>
                <h3 className="text-base font-bold text-white">Deterministic Rule Engine</h3>
                <p className="mt-2 text-sm text-slate-300 leading-relaxed">
                  Evaluates velocity thresholds across 5m, 1h, and 24h sliding windows with pure integer precision. No floating point anomalies in compliance decisions.
                </p>
              </div>

              <div className="rounded-xl border border-slate-800 bg-slate-900/50 p-6">
                <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-cyan-500/10 text-cyan-400 border border-cyan-500/20 mb-4">
                  <Cpu className="h-5 w-5" />
                </div>
                <h3 className="text-base font-bold text-white">Embedded Machine Learning</h3>
                <p className="mt-2 text-sm text-slate-300 leading-relaxed">
                  Trained on quantitative fraud distributions and executed directly inside Rust with ONNX parity. Emits risk scores in basis points without network roundtrips.
                </p>
              </div>

              <div className="rounded-xl border border-slate-800 bg-slate-900/50 p-6">
                <div className="flex h-10 w-10 items-center justify-center rounded-lg bg-teal-500/10 text-teal-400 border border-teal-500/20 mb-4">
                  <History className="h-5 w-5" />
                </div>
                <h3 className="text-base font-bold text-white">Tamper-Evident Audit Chain</h3>
                <p className="mt-2 text-sm text-slate-300 leading-relaxed">
                  Every decision outcome is cryptographically linked with predecessor hash chaining. Reconstructable feature and rule versions satisfy strict regulatory audits.
                </p>
              </div>
            </div>
          </div>
        </div>
      </main>

      {/* Footer */}
      <footer className="border-t border-slate-800/80 py-8 px-6 text-center text-xs text-slate-400 bg-slate-950">
        <p>&copy; 2026 CapitalCore Financial Technologies. All rights reserved. High-performance banking intelligence.</p>
      </footer>
    </div>
  );
}
