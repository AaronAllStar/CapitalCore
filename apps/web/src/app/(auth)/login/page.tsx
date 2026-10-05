"use client";

import { useEffect } from "react";
import { useRouter } from "next/navigation";
import { RotateCw, Building2 } from "lucide-react";

export default function LoginPage() {
  const router = useRouter();

  useEffect(() => {
    router.replace("/dashboard");
  }, [router]);

  return (
    <div className="flex min-h-screen items-center justify-center p-4 bg-slate-950 text-slate-100">
      <div className="flex flex-col items-center gap-3 font-mono text-xs text-slate-400">
        <div className="flex h-12 w-12 items-center justify-center rounded-xl bg-emerald-500/15 border border-emerald-500/30 text-emerald-400">
          <Building2 className="h-6 w-6" />
        </div>
        <div className="flex items-center gap-2">
          <RotateCw className="h-3.5 w-3.5 animate-spin text-emerald-400" />
          <span>Entering CapitalCore Banking Operations Console...</span>
        </div>
      </div>
    </div>
  );
}
