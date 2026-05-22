import { useEffect, useState } from "react";
import { Database, FolderOpen, HardDriveDownload, CheckCircle2, AlertTriangle } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { backupDir, createBackup } from "@/lib/ipc";
import type { BackupResult, HealthReport } from "@/lib/types";

function formatBytes(n: number): string {
  if (n < 1024) return `${n} B`;
  const units = ["KB", "MB", "GB"];
  let v = n / 1024;
  let i = 0;
  while (v >= 1024 && i < units.length - 1) {
    v /= 1024;
    i++;
  }
  return `${v.toFixed(1)} ${units[i]}`;
}

/**
 * Full database backup. Non-destructive, so no confirmation — but it's also the
 * engine the (upcoming) restore reuses as its automatic safety backup. Gated on
 * the live "db" health check.
 */
export function BackupPage({ health }: { health: HealthReport | null }) {
  const dbReady = health?.checks.find((c) => c.id === "db")?.status === "ok";
  const [dir, setDir] = useState("");
  const [busy, setBusy] = useState(false);
  const [result, setResult] = useState<BackupResult | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    void backupDir().then(setDir);
  }, []);

  const run = async () => {
    setBusy(true);
    setResult(null);
    setError(null);
    try {
      setResult(await createBackup());
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="flex flex-col gap-4">
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <HardDriveDownload className="h-4 w-4 text-brand" />
            Back up your databases
          </CardTitle>
        </CardHeader>
        <CardContent className="flex flex-col gap-3">
          <p className="text-sm text-slate-300">
            Saves a complete copy of your accounts, characters and world data to a single file you
            can restore later.
          </p>
          <div className="flex items-center gap-2 text-xs text-slate-400">
            <FolderOpen className="h-3.5 w-3.5" />
            <span className="truncate">{dir || "…"}</span>
            <span className="text-slate-600">(change in Settings)</span>
          </div>

          {!dbReady && (
            <p className="text-xs text-amber-300">
              Your database isn't reachable right now — check the Status page first.
            </p>
          )}

          <div className="flex items-center gap-3">
            <Button onClick={run} disabled={busy || !dbReady}>
              {busy ? "Backing up…" : "Back up now"}
            </Button>
            {busy && (
              <span className="text-xs text-slate-400">
                This can take a minute — the world database is large.
              </span>
            )}
          </div>

          {error && (
            <p className="flex items-center gap-1.5 text-sm text-rose-300">
              <AlertTriangle className="h-4 w-4" /> {error}
            </p>
          )}
        </CardContent>
      </Card>

      {result && <BackupSummary result={result} />}
    </div>
  );
}

function BackupSummary({ result }: { result: BackupResult }) {
  return (
    <Card className={result.completed ? "border-emerald-500/40" : "border-amber-500/40"}>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          {result.completed ? (
            <CheckCircle2 className="h-4 w-4 text-emerald-400" />
          ) : (
            <AlertTriangle className="h-4 w-4 text-amber-400" />
          )}
          {result.completed ? "Backup complete" : "Backup finished — please double-check it"}
        </CardTitle>
      </CardHeader>
      <CardContent className="flex flex-col gap-3 text-sm">
        <div className="grid grid-cols-3 gap-3 text-center">
          <Stat label="Size" value={formatBytes(result.sizeBytes)} />
          <Stat label="Databases" value={String(result.databases.length)} />
          <Stat label="Took" value={`${result.durationSecs}s`} />
        </div>

        {!result.completed && (
          <p className="text-xs text-amber-300">
            The file didn't end with the expected completion marker, so it may be incomplete. Try
            again, and check there's enough free disk space.
          </p>
        )}

        <div className="rounded-lg border border-slate-800">
          <div className="grid grid-cols-3 gap-2 border-b border-slate-800 px-3 py-2 text-xs font-medium text-slate-400">
            <span>Database</span>
            <span className="text-right">Tables</span>
            <span className="text-right">Rows (approx.)</span>
          </div>
          {result.databases.map((d) => (
            <div key={d.name} className="grid grid-cols-3 gap-2 px-3 py-2 text-xs">
              <span className="flex items-center gap-1.5">
                <Database className="h-3.5 w-3.5 text-slate-500" />
                {d.name}
              </span>
              <span className="text-right tabular-nums">{d.tables.toLocaleString()}</span>
              <span className="text-right tabular-nums">{d.approxRows.toLocaleString()}</span>
            </div>
          ))}
        </div>

        <p className="truncate text-xs text-slate-500" title={result.path}>
          Saved to {result.path}
        </p>
      </CardContent>
    </Card>
  );
}

function Stat({ label, value }: { label: string; value: string }) {
  return (
    <div className="rounded-lg bg-slate-800/40 py-2">
      <div className="text-lg font-semibold text-brand-glow">{value}</div>
      <div className="text-xs text-slate-400">{label}</div>
    </div>
  );
}
