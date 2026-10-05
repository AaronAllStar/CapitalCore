"use client";

import { DecisionType } from "@/types";
import { CheckCircle2, AlertTriangle, AlertOctagon, XCircle, Globe, Smartphone, CreditCard, Landmark, Terminal } from "lucide-react";
import { cn } from "@/lib/utils";

interface DecisionBadgeProps {
  decision: DecisionType | string;
  className?: string;
  showIcon?: boolean;
}

export function DecisionBadge({ decision, className, showIcon = true }: DecisionBadgeProps) {
  const norm = (decision || "").toUpperCase();

  switch (norm) {
    case "ALLOW":
      return (
        <span
          className={cn(
            "inline-flex items-center gap-1.5 rounded-full px-2.5 py-0.5 text-xs font-semibold border bg-emerald-500/15 text-emerald-300 border-emerald-500/30",
            className
          )}
        >
          {showIcon && <CheckCircle2 className="h-3.5 w-3.5 text-emerald-400" />}
          ALLOW
        </span>
      );
    case "REVIEW":
      return (
        <span
          className={cn(
            "inline-flex items-center gap-1.5 rounded-full px-2.5 py-0.5 text-xs font-semibold border bg-amber-500/15 text-amber-300 border-amber-500/30",
            className
          )}
        >
          {showIcon && <AlertTriangle className="h-3.5 w-3.5 text-amber-400" />}
          REVIEW
        </span>
      );
    case "ESCALATE":
      return (
        <span
          className={cn(
            "inline-flex items-center gap-1.5 rounded-full px-2.5 py-0.5 text-xs font-semibold border bg-orange-500/15 text-orange-300 border-orange-500/30",
            className
          )}
        >
          {showIcon && <AlertOctagon className="h-3.5 w-3.5 text-orange-400" />}
          ESCALATE
        </span>
      );
    case "BLOCK":
      return (
        <span
          className={cn(
            "inline-flex items-center gap-1.5 rounded-full px-2.5 py-0.5 text-xs font-semibold border bg-rose-500/15 text-rose-300 border-rose-500/30",
            className
          )}
        >
          {showIcon && <XCircle className="h-3.5 w-3.5 text-rose-400" />}
          BLOCK
        </span>
      );
    default:
      return (
        <span
          className={cn(
            "inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-medium border bg-slate-800 text-slate-300 border-slate-700",
            className
          )}
        >
          {decision}
        </span>
      );
  }
}

interface MoneyDisplayProps {
  amountMinor?: number;
  amount?: number;
  currency?: string;
  className?: string;
}

export function MoneyDisplay({ amountMinor, amount, currency = "USD", className }: MoneyDisplayProps) {
  const minorVal = amountMinor !== undefined ? amountMinor : Math.round((amount ?? 0) * 100);
  const formatted = new Intl.NumberFormat("en-US", {
    style: "currency",
    currency: currency.toUpperCase(),
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  }).format(minorVal / 100);

  return (
    <span className={cn("font-mono font-medium tracking-tight", className)}>
      {formatted}
    </span>
  );
}

interface ChannelBadgeProps {
  channel: string;
}

export function ChannelBadge({ channel }: ChannelBadgeProps) {
  const norm = (channel || "").toUpperCase();
  let icon = <Globe className="h-3 w-3" />;

  if (norm.includes("MOBILE") || norm.includes("IOS") || norm.includes("ANDROID")) {
    icon = <Smartphone className="h-3 w-3" />;
  } else if (norm.includes("POS") || norm.includes("CARD")) {
    icon = <CreditCard className="h-3 w-3" />;
  } else if (norm.includes("ATM")) {
    icon = <Landmark className="h-3 w-3" />;
  } else if (norm.includes("API")) {
    icon = <Terminal className="h-3 w-3" />;
  }

  return (
    <span className="inline-flex items-center gap-1 text-[11px] font-medium text-slate-400 bg-slate-800/60 border border-slate-700/60 rounded px-2 py-0.5">
      {icon}
      <span>{norm}</span>
    </span>
  );
}
