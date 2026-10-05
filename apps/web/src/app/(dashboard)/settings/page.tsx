"use client";

import { useState } from "react";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Separator } from "@/components/ui/separator";
import { useAuthStore } from "@/stores/auth";
import { ShieldCheck, UserCheck, Key, Lock, CheckCircle2 } from "lucide-react";

export default function SettingsPage() {
  const { user } = useAuthStore();
  const [displayName, setDisplayName] = useState(user?.name || "Lead Risk Analyst");
  const [saved, setSaved] = useState(false);

  const handleSave = () => {
    setSaved(true);
    setTimeout(() => setSaved(false), 3000);
  };

  return (
    <div className="max-w-4xl mx-auto space-y-6">
      <div>
        <h1 className="text-2xl font-bold tracking-tight text-foreground">Analyst Profile & System Settings</h1>
        <p className="text-sm text-muted-foreground">Manage your credentials, active role permissions, and session tokens.</p>
      </div>

      {saved && (
        <div className="p-3 rounded-lg border border-emerald-500/30 bg-emerald-500/10 text-xs text-emerald-400 font-mono flex items-center gap-2">
          <CheckCircle2 className="h-4 w-4" />
          <span>Analyst preferences updated successfully.</span>
        </div>
      )}

      {/* Analyst Profile */}
      <Card>
        <CardHeader>
          <CardTitle className="text-base flex items-center gap-2">
            <UserCheck className="h-4 w-4 text-primary" />
            Security & Identity Profile
          </CardTitle>
          <CardDescription>Enterprise banking role assignments and department metadata.</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid gap-4 sm:grid-cols-2">
            <div className="space-y-2">
              <Label>Full Name / Title</Label>
              <Input
                value={displayName}
                onChange={(e) => setDisplayName(e.target.value)}
                className="font-mono text-sm"
              />
            </div>
            <div className="space-y-2">
              <Label>Corporate Email</Label>
              <Input value={user?.email || "analyst@capitalcore.bank"} disabled className="font-mono text-sm bg-muted/40" />
            </div>
            <div className="space-y-2">
              <Label>Assigned Role</Label>
              <Input value={user?.role || "Risk & Compliance Officer"} disabled className="font-mono text-sm bg-muted/40" />
            </div>
            <div className="space-y-2">
              <Label>Department</Label>
              <Input value={user?.department || "Global Fraud Operations"} disabled className="font-mono text-sm bg-muted/40" />
            </div>
          </div>
          <Button onClick={handleSave} className="text-xs">Save Changes</Button>
        </CardContent>
      </Card>

      {/* RBAC & Cryptographic Scope */}
      <Card>
        <CardHeader>
          <CardTitle className="text-base flex items-center gap-2">
            <ShieldCheck className="h-4 w-4 text-primary" />
            Authorized RBAC Permissions
          </CardTitle>
          <CardDescription>Scoped permissions encoded into your Ed25519 session claims.</CardDescription>
        </CardHeader>
        <CardContent className="space-y-3">
          <div className="grid gap-2 sm:grid-cols-2">
            {(user?.permissions || [
              "events:read",
              "events:write",
              "decisions:read",
              "rules:read",
              "audit:read",
            ]).map((perm) => (
              <div key={perm} className="flex items-center gap-2 rounded-lg border border-border/60 bg-muted/20 p-2.5 font-mono text-xs">
                <Lock className="h-3.5 w-3.5 text-emerald-400" />
                <span className="font-semibold text-foreground">{perm}</span>
                <span className="ml-auto text-[10px] text-emerald-400 uppercase">Granted</span>
              </div>
            ))}
          </div>
        </CardContent>
      </Card>

      {/* API Connectivity */}
      <Card>
        <CardHeader>
          <CardTitle className="text-base flex items-center gap-2">
            <Key className="h-4 w-4 text-primary" />
            Core Decision API Binding
          </CardTitle>
          <CardDescription>Network endpoints and latency telemetry.</CardDescription>
        </CardHeader>
        <CardContent className="space-y-2 text-xs font-mono">
          <div className="flex justify-between py-2 border-b border-border/40">
            <span className="text-muted-foreground">API Host Endpoint</span>
            <span className="text-foreground font-semibold">http://localhost:8001/api/v1</span>
          </div>
          <div className="flex justify-between py-2 border-b border-border/40">
            <span className="text-muted-foreground">Frontend Console Host</span>
            <span className="text-foreground font-semibold">http://localhost:3001</span>
          </div>
          <div className="flex justify-between py-2">
            <span className="text-muted-foreground">Signature Algorithm</span>
            <span className="text-foreground font-semibold">Ed25519 (RFC 8032)</span>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
