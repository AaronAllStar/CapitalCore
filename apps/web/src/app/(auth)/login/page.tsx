"use client";

import { useEffect, useState } from "react";
import { useRouter } from "next/navigation";
import { ShieldCheck, ArrowRight, Building2, Lock, RotateCw } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { useAuthStore } from "@/stores/auth";

export default function LoginPage() {
  const router = useRouter();
  const { loginAsAnalyst, user, isAuthenticated } = useAuthStore();
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    // If already authenticated, redirect straight to dashboard
    if (isAuthenticated && user) {
      router.replace("/dashboard");
    }
  }, [isAuthenticated, user, router]);

  async function handleAccess() {
    setLoading(true);
    try {
      await loginAsAnalyst();
      router.push("/dashboard");
    } finally {
      setLoading(false);
    }
  }

  return (
    <div className="flex min-h-screen items-center justify-center p-4 bg-background relative overflow-hidden">
      <div className="absolute inset-0 bg-[radial-gradient(circle_at_top,_var(--tw-gradient-stops))] from-primary/10 via-background to-background" />

      <Card className="w-full max-w-md border-border/80 bg-card/90 backdrop-blur-md shadow-2xl relative z-10">
        <CardHeader className="text-center pb-4">
          <div className="mx-auto mb-3 flex h-12 w-12 items-center justify-center rounded-xl bg-primary/15 border border-primary/30">
            <Building2 className="h-6 w-6 text-primary" />
          </div>
          <CardTitle className="text-2xl font-bold tracking-tight text-foreground">
            CapitalCore Operations
          </CardTitle>
          <CardDescription className="text-muted-foreground text-xs uppercase tracking-wider font-mono">
            Enterprise Banking Risk &amp; Fraud Console
          </CardDescription>
        </CardHeader>

        <CardContent className="space-y-5">
          <div className="rounded-lg border border-emerald-500/20 bg-emerald-500/5 p-4 text-xs text-muted-foreground space-y-2">
            <div className="flex items-center gap-1.5 font-semibold text-emerald-400">
              <ShieldCheck className="h-4 w-4" />
              <span>Passwordless Cryptographic Authentication</span>
            </div>
            <p className="text-foreground/80 leading-relaxed">
              Access is pre-authorized for Bank Risk Analysts and Fraud Officers via asymmetric Ed25519 JWT session issuance. Login/Signup por correo y contraseñas tradicionales ha sido desactivado.
            </p>
          </div>

          <div className="p-3 rounded-lg border border-border/60 bg-muted/20 space-y-1.5 text-xs font-mono">
            <div className="flex justify-between text-muted-foreground">
              <span>Security Context:</span>
              <span className="text-foreground font-semibold">SOX-404 / Basel III</span>
            </div>
            <div className="flex justify-between text-muted-foreground">
              <span>Default Principal:</span>
              <span className="text-foreground font-semibold">Lead Risk Analyst</span>
            </div>
            <div className="flex justify-between text-muted-foreground">
              <span>Signature Suite:</span>
              <span className="text-primary font-semibold">Ed25519 · SHA-256</span>
            </div>
          </div>

          <Button
            type="button"
            className="w-full h-11 font-semibold flex items-center justify-center gap-2 shadow-lg text-sm bg-primary text-primary-foreground hover:bg-primary/90"
            onClick={handleAccess}
            disabled={loading}
          >
            {loading ? (
              <>
                <RotateCw className="h-4 w-4 animate-spin" />
                Estableciendo sesión criptográfica...
              </>
            ) : (
              <>
                <Lock className="h-4 w-4" />
                Ingresar a Consola Operativa
                <ArrowRight className="h-4 w-4 ml-auto" />
              </>
            )}
          </Button>

          <p className="text-center text-[11px] text-muted-foreground font-mono">
            Audit logging active • Ledger tamper-evident
          </p>
        </CardContent>
      </Card>
    </div>
  );
}
