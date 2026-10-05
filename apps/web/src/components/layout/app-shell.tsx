"use client";

import { useEffect } from "react";
import { useAuthStore } from "@/stores/auth";
import { Sidebar } from "@/components/layout/sidebar";
import { Header } from "@/components/layout/header";

export function AppShell({ children }: { children: React.ReactNode }) {
  const { fetchSession } = useAuthStore();

  useEffect(() => {
    fetchSession().catch(() => {});
  }, [fetchSession]);

  return (
    <div className="min-h-screen bg-slate-950 text-slate-100">
      <Sidebar />
      <div className="pl-64">
        <Header />
        <main className="p-6">{children}</main>
      </div>
    </div>
  );
}
