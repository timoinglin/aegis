import { useEffect, useState } from "react";
import { Wrench, BarChart3, Sparkles, Stethoscope, CheckCircle2, AlertTriangle, Database, RotateCw } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { dbMaintenance, dbSizes } from "@/lib/ipc";
import type { DbSize, HealthReport, MaintenanceResult } from "@/lib/types";

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
  const [sizes, setSizes] = useState<DbSize[] | null>(null);
  const [sizesBusy, setSizesBusy] = useState(false);

  const refreshSizes = async () => {
    if (!dbReady) return;
    setSizesBusy(true);
    try {
      setSizes(await dbSizes());
    } catch {
      /* shown as "—" rows */
    } finally {
      setSizesBusy(false);
    }
  };

  useEffect(() => {
    void refreshSizes();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [dbReady]);

  const run = async (mode: Mode) => {
    setBusy(mode);
    setResult(null);
    setError(null);
    try {
      setResult(await dbMaintenance(mode));
      // After optimize, sizes can change — re-pull them.
      if (mode === "optimize") void refreshSizes();
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

      <Card>
        <CardHeader className="flex flex-row items-center justify-between">
          <CardTitle className="flex items-center gap-2">
            <Database className="h-4 w-4 text-brand" />
            Database sizes
          </CardTitle>
          <Button variant="ghost" size="icon" disabled={sizesBusy || !dbReady} onClick={refreshSizes} title="Refresh">
            <RotateCw className={`h-3.5 w-3.5 ${sizesBusy ? "animate-spin" : ""}`} />
          </Button>
        </CardHeader>
        <CardContent>
          {!sizes && !dbReady && (
            <p className="text-xs text-slate-500">Connect to your database to see sizes.</p>
          )}
          {sizes && sizes.length === 0 && (
            <p className="text-xs text-slate-500">No databases found.</p>
          )}
          {sizes && sizes.length > 0 && (
            <div className="overflow-hidden rounded-md border border-slate-800">
              <table className="w-full text-sm">
                <thead className="bg-slate-900/40 text-xs uppercase tracking-wide text-slate-400">
                  <tr>
                    <th className="px-3 py-2 text-left">Database</th>
                    <th className="px-3 py-2 text-right">Tables</th>
                    <th className="px-3 py-2 text-right">Size on disk</th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-slate-800">
                  {sizes.map((s) => (
                    <tr key={s.name}>
                      <td className="px-3 py-2 font-medium text-slate-200">{s.name}</td>
                      <td className="px-3 py-2 text-right text-slate-300">{s.tableCount.toLocaleString()}</td>
                      <td className="px-3 py-2 text-right text-slate-300">{formatBytes(s.sizeBytes)}</td>
                    </tr>
                  ))}
                  <tr className="bg-slate-900/30 font-semibold">
                    <td className="px-3 py-2 text-slate-100">Total</td>
                    <td className="px-3 py-2 text-right text-slate-200">
                      {sizes.reduce((n, s) => n + s.tableCount, 0).toLocaleString()}
                    </td>
                    <td className="px-3 py-2 text-right text-slate-100">
                      {formatBytes(sizes.reduce((n, s) => n + s.sizeBytes, 0))}
                    </td>
                  </tr>
                </tbody>
              </table>
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}

function formatBytes(n: number): string {
  if (n < 1024) return `${n} B`;
  const units = ["KB", "MB", "GB", "TB"];
  let v = n / 1024;
  let i = 0;
  while (v >= 1024 && i < units.length - 1) {
    v /= 1024;
    i++;
  }
  return `${v.toFixed(v >= 100 ? 0 : v >= 10 ? 1 : 2)} ${units[i]}`;
}
