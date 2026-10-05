"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import {
  LayoutDashboard,
  Receipt,
  PlayCircle,
  AlertTriangle,
  History,
  Cpu,
  Activity,
  ShieldCheck,
  LogOut,
  Landmark,
} from "lucide-react";
import { cn } from "@/lib/utils";
import { useAuthStore } from "@/stores/auth";

const navItems = [
  { href: "/dashboard", label: "Dashboard", icon: LayoutDashboard },
  { href: "/decisions", label: "Decisions Stream", icon: Receipt },
  { href: "/simulate", label: "Simulator Studio", icon: PlayCircle },
  { href: "/review", label: "Review Queue", icon: AlertTriangle },
  { href: "/audit", label: "Audit & Integrity", icon: History },
  { href: "/model", label: "ML Model Card", icon: Cpu },
  { href: "/system", label: "System Health", icon: Activity },
];

export function Sidebar() {
  const pathname = usePathname();
  const { user, logout } = useAuthStore();

  return (
    <aside className="fixed left-0 top-0 z-40 flex h-screen w-64 flex-col border-r border-border bg-slate-950 text-slate-100">
      {/* Brand Header */}
      <div className="flex h-16 items-center gap-3 border-b border-slate-800/80 px-6 bg-slate-900/60 backdrop-blur-md">
        <div className="flex h-9 w-9 items-center justify-center rounded-lg bg-emerald-500/20 text-emerald-400 border border-emerald-500/30">
          <Landmark className="h-5 w-5" />
        </div>
        <div>
          <div className="flex items-center gap-1.5">
            <span className="text-base font-bold tracking-tight text-white">CapitalCore</span>
            <span className="rounded bg-emerald-500/20 px-1.5 py-0.2 text-[10px] font-semibold text-emerald-300 border border-emerald-500/30">
              BANKING
            </span>
          </div>
          <p className="text-[11px] text-slate-400">Decision Intelligence</p>
        </div>
      </div>

      {/* Navigation */}
      <nav className="flex-1 space-y-1 px-3 py-5 overflow-y-auto">
        <div className="px-3 pb-2 text-[11px] font-semibold uppercase tracking-wider text-slate-400">
          Fraud & Policy Engine
        </div>
        {navItems.map((item) => {
          const active = pathname === item.href || (item.href !== "/dashboard" && pathname.startsWith(item.href));
          return (
            <Link
              key={item.href}
              href={item.href}
              className={cn(
                "flex items-center gap-3 rounded-lg px-3 py-2.5 text-sm font-medium transition-all",
                active
                  ? "bg-emerald-500/15 text-emerald-300 border border-emerald-500/30 shadow-sm"
                  : "text-slate-300 hover:bg-slate-800/60 hover:text-white"
              )}
            >
              <item.icon className={cn("h-4 w-4", active ? "text-emerald-400" : "text-slate-400")} />
              {item.label}
            </Link>
          );
        })}
      </nav>

      {/* Analyst profile badge */}
      {user && (
        <div className="border-t border-slate-800/80 p-4 bg-slate-900/40">
          <div className="flex items-center gap-3">
            <div className="flex h-9 w-9 items-center justify-center rounded-full bg-slate-800 text-xs font-bold text-emerald-400 border border-slate-700">
              <ShieldCheck className="h-5 w-5 text-emerald-400" />
            </div>
            <div className="flex-1 overflow-hidden">
              <p className="truncate text-xs font-semibold text-slate-200">
                {user.name}
              </p>
              <p className="truncate text-[11px] text-slate-400">
                {user.role}
              </p>
            </div>
            <button
              onClick={logout}
              title="Sign out"
              className="rounded-md p-1.5 text-slate-400 transition-colors hover:bg-slate-800 hover:text-rose-400"
            >
              <LogOut className="h-4 w-4" />
            </button>
          </div>
        </div>
      )}
    </aside>
  );
}
