import { useState } from "react";
import { Wrench, BarChart3, Sparkles, Stethoscope, CheckCircle2, AlertTriangle } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { dbMaintenance } from "@/lib/ipc";
import type { HealthReport, MaintenanceResult } from "@/lib/types";

type Mode = "analyze" | "optimize" | "repair";

const ACTIONS: { mode: Mode; label: string; icon: React.ReactNode; blurb: string }[] = [
  {
    mode: "analyze",
    label: "Analyze",
    icon: <BarChart3 className="h-4 w-4 text-brand" />,
    blurb: "Refreshes the database's internal statistics so queries stay fast. Quick and safe.",
  },
  {
    mode: "optimize",
    label: "Optimize",
    icon: <Sparkles className="h-4 w-4 text-brand" />,
    blurb: "Tidies up tables and reclaims unused space. Can take a while on the large world database.",
  },
  {
    mode: "repair",
    label: "Check & repair",
    icon: <Stethoscope className="h-4 w-4 text-brand" />,
    blurb: "Checks every table for problems and repairs what it can. (Most repack tables are InnoDB, which already self-heal — so this is mainly a health check.)",
  },
];

export function MaintenancePage({ health }: { health: HealthReport | null }) {
  const dbReady = health?.checks.find((c) => c.id === "db")?.status === "ok";
  const [busy, setBusy] = useState<Mode | null>(null);
  const [result, setResult] = useState<MaintenanceResult | null>(null);
  const [error, setError] = useState<string | null>(null);

  const run = async (mode: Mode) => {
    setBusy(mode);
    setResult(null);
    setError(null);
    try {
      setResult(await dbMaintenance(mode));
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(null);
    }
  };

  return (
    <div className="flex flex-col gap-4">
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Wrench className="h-4 w-4 text-brand" />
            Database maintenance
          </CardTitle>
        </CardHeader>
        <CardContent className="flex flex-col gap-3">
          <p className="text-sm text-slate-300">
            Routine upkeep to keep your server's database fast and healthy. Best run when the server
            is quiet.
          </p>
          {!dbReady && (
            <p className="text-xs text-amber-300">
              Your database isn't reachable right now — check the Status page first.
            </p>
          )}
          <div className="flex flex-col divide-y divide-slate-800 rounded-lg border border-slate-800">
            {ACTIONS.map((a) => (
              <div key={a.mode} className="flex items-start gap-3 p-3">
                <div className="mt-0.5 shrink-0">{a.icon}</div>
                <div className="flex-1">
                  <p className="text-sm font-medium">{a.label}</p>
                  <p className="mt-0.5 text-xs text-slate-400">{a.blurb}</p>
                </div>
                <Button variant="outline" size="sm" className="shrink-0" disabled={!!busy || !dbReady} onClick={() => run(a.mode)}>
                  {busy === a.mode ? "Working…" : a.label}
                </Button>
              </div>
            ))}
          </div>

          {error && (
            <p className="flex items-start gap-1.5 text-sm text-rose-300">
              <AlertTriangle className="mt-0.5 h-4 w-4 shrink-0" /> {error}
            </p>
          )}
        </CardContent>
      </Card>

      {result && (
        <Card className={result.healthy ? "border-emerald-500/40" : "border-amber-500/40"}>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              {result.healthy ? (
                <CheckCircle2 className="h-4 w-4 text-emerald-400" />
              ) : (
                <AlertTriangle className="h-4 w-4 text-amber-400" />
              )}
              Result
            </CardTitle>
          </CardHeader>
          <CardContent className="flex flex-col gap-2">
            <p className="text-sm text-slate-200">{result.message}</p>
            {result.details && (
              <pre className="max-h-60 overflow-auto whitespace-pre-wrap rounded-md bg-slate-950 p-2 text-xs text-slate-400">
                {result.details}
              </pre>
            )}
          </CardContent>
        </Card>
      )}
    </div>
  );
}
