"use client";

import Link from "next/link";
import { Badge } from "@/components/ui/badge";
import { PlayCircle, ShieldCheck, Database, CheckCircle2 } from "lucide-react";

export function Header() {
  return (
    <header className="sticky top-0 z-30 flex h-16 items-center justify-between border-b border-border bg-slate-900/80 px-6 backdrop-blur-md">
      {/* Left: Engine status & Environment */}
      <div className="flex items-center gap-3">
        <div className="flex items-center gap-2 rounded-full border border-emerald-500/30 bg-emerald-500/10 px-3 py-1 text-xs font-medium text-emerald-400">
          <span className="relative flex h-2 w-2">
            <span className="absolute inline-flex h-full w-full animate-ping rounded-full bg-emerald-400 opacity-75"></span>
            <span className="relative inline-flex h-2 w-2 rounded-full bg-emerald-500"></span>
          </span>
          <span>Core Decision Engine: Active</span>
        </div>

        <div className="hidden sm:flex items-center gap-1.5 text-xs text-slate-400 border border-slate-800 bg-slate-950/60 rounded-full px-3 py-1">
          <Database className="h-3 w-3 text-cyan-400" />
          <span>PostgreSQL Audit Store</span>
        </div>
      </div>

      {/* Right: Model status, Quick simulate button */}
      <div className="flex items-center gap-3">
        <Badge variant="outline" className="hidden md:flex border-emerald-500/30 bg-emerald-950/30 text-emerald-400 font-mono text-[11px] gap-1.5 py-1">
          <CheckCircle2 className="h-3 w-3 text-emerald-400" />
          ML Model: 99.6% Parity
        </Badge>

        <Link
          href="/simulate"
          className="flex items-center gap-1.5 rounded-lg bg-emerald-600 px-3.5 py-1.5 text-xs font-semibold text-white shadow-sm hover:bg-emerald-500 transition-colors"
        >
          <PlayCircle className="h-3.5 w-3.5" />
          <span>Simulate Transaction</span>
        </Link>
      </div>
    </header>
  );
}
