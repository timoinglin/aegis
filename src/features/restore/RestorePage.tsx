import { useEffect, useState } from "react";
import { AlertTriangle, CheckCircle2, RotateCcw, ServerCrash, ShieldCheck, Database } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { listBackups, restoreBackup, serverStatus } from "@/lib/ipc";
import type { BackupFile, HealthReport, RestoreResult, ServerStatus } from "@/lib/types";

const CONFIRM_WORD = "RESTORE";

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
 * Restore a backup — the destructive operation. The backend enforces the full
 * safety net (auto safety-backup, typed confirmation, server-stopped guard); the
 * UI mirrors those guards so the user understands what's happening before they act.
 */
// `health` left in the props for now in case future checks need it.
// We deliberately don't read serverRunning from it — Health is cached on app
// load and quickly goes stale, which is why the radio used to stick disabled
// after stopping the server. Instead we poll the live process state.
// eslint-disable-next-line @typescript-eslint/no-unused-vars
export function RestorePage({ health: _health }: { health: HealthReport | null }) {
  const [status, setStatus] = useState<ServerStatus | null>(null);
  const [backups, setBackups] = useState<BackupFile[]>([]);
  const [selected, setSelected] = useState<string | null>(null);
  const [confirm, setConfirm] = useState("");
  const [busy, setBusy] = useState(false);
  const [result, setResult] = useState<RestoreResult | null>(null);
  const [error, setError] = useState<string | null>(null);

  // Live worldserver/authserver state — restore is blocked if either is up.
  const refreshStatus = async () => {
    try {
      setStatus(await serverStatus());
    } catch {
      /* a transient tasklist failure isn't fatal — next tick retries */
    }
  };

  useEffect(() => {
    void listBackups().then(setBackups);
    void refreshStatus();
    const t = setInterval(refreshStatus, 3000);
    return () => clearInterval(t);
  }, []);

  const serverRunning = (status?.worldserver.running ?? false) || (status?.authserver.running ?? false);
  const canRestore = !!selected && confirm.trim() === CONFIRM_WORD && !serverRunning && !busy;

  const run = async () => {
    if (!selected) return;
    setBusy(true);
    setResult(null);
    setError(null);
    try {
      setResult(await restoreBackup(selected, confirm.trim()));
      setConfirm("");
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="flex flex-col gap-4">
      <Card className="border-rose-500/30">
        <CardContent className="flex items-start gap-3 py-3">
          <AlertTriangle className="mt-0.5 h-5 w-5 shrink-0 text-rose-400" />
          <p className="text-sm text-slate-300">
            Restoring <span className="font-medium text-rose-300">replaces your current accounts,
            characters and world data</span> with the contents of the backup. Aegis automatically
            takes a fresh safety backup first, so you can always go back.
          </p>
        </CardContent>
      </Card>

      {serverRunning && (
        <Card className="border-rose-500/40">
          <CardContent className="flex items-start gap-3 py-3">
            <ServerCrash className="mt-0.5 h-5 w-5 shrink-0 text-rose-400" />
            <div>
              <p className="text-sm font-medium text-rose-300">
                Stop your server before restoring
                {status && (
                  <span className="ml-1 text-xs font-normal text-rose-400">
                    ({[
                      status.worldserver.running ? "worldserver" : null,
                      status.authserver.running ? "authserver" : null,
                    ].filter(Boolean).join(" + ")} running)
                  </span>
                )}
              </p>
              <p className="mt-0.5 text-xs text-slate-400">
                Restoring while the worldserver or authserver is running can corrupt your database.
                Stop them from the Server tab — Aegis re-checks every few seconds.
              </p>
            </div>
          </CardContent>
        </Card>
      )}

      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <RotateCcw className="h-4 w-4 text-brand" />
            Choose a backup to restore
          </CardTitle>
        </CardHeader>
        <CardContent className="flex flex-col gap-3">
          {backups.length === 0 ? (
            <p className="text-sm text-slate-400">
              No backups found yet. Take one from the Backup page first.
            </p>
          ) : (
            <div className="flex flex-col divide-y divide-slate-800 rounded-lg border border-slate-800">
              {backups.map((b) => (
                <label
                  key={b.path}
                  className={`flex cursor-pointer items-center gap-3 px-3 py-2 text-sm ${
                    selected === b.path ? "bg-brand/10" : "hover:bg-slate-800/50"
                  }`}
                >
                  <input
                    type="radio"
                    name="backup"
                    className="accent-brand"
                    checked={selected === b.path}
                    onChange={() => setSelected(b.path)}
                    disabled={serverRunning}
                  />
                  <span className="flex-1 truncate">{b.name}</span>
                  <span className="text-xs text-slate-500">{formatBytes(b.sizeBytes)}</span>
                  <span className="text-xs text-slate-500">
                    {new Date(b.modifiedMs).toLocaleString()}
                  </span>
                </label>
              ))}
            </div>
          )}

          {selected && !serverRunning && (
            <div className="flex flex-col gap-2 rounded-lg bg-slate-800/30 p-3">
              <label className="text-sm text-slate-300">
                Type <span className="font-semibold text-rose-300">{CONFIRM_WORD}</span> to confirm
                you want to replace your current data:
              </label>
              <input
                className="w-48 rounded-md border border-slate-700 bg-slate-950 px-3 py-1.5 text-sm outline-none focus:border-brand"
                value={confirm}
                placeholder={CONFIRM_WORD}
                onChange={(e) => setConfirm(e.target.value)}
              />
            </div>
          )}

          <div className="flex items-center gap-3">
            <Button onClick={run} disabled={!canRestore}>
              {busy ? "Restoring…" : "Restore this backup"}
            </Button>
            {busy && (
              <span className="text-xs text-slate-400">
                Taking a safety backup first, then restoring — don't close Aegis.
              </span>
            )}
          </div>

          {error && (
            <p className="flex items-start gap-1.5 text-sm text-rose-300">
              <AlertTriangle className="mt-0.5 h-4 w-4 shrink-0" /> {error}
            </p>
          )}
        </CardContent>
      </Card>

      {result && <RestoreSummary result={result} />}
    </div>
  );
}

function RestoreSummary({ result }: { result: RestoreResult }) {
  return (
    <Card className="border-emerald-500/40">
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <CheckCircle2 className="h-4 w-4 text-emerald-400" />
          Restore complete
        </CardTitle>
      </CardHeader>
      <CardContent className="flex flex-col gap-3 text-sm">
        <div className="flex items-start gap-2 rounded-lg bg-slate-800/30 p-3 text-xs">
          <ShieldCheck className="mt-0.5 h-4 w-4 shrink-0 text-brand" />
          <span className="text-slate-300">
            A safety backup of your previous data was saved first, at{" "}
            <span className="text-slate-200">{result.safetyBackupPath}</span> — restore it if you
            need to undo this.
          </span>
        </div>

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
      </CardContent>
    </Card>
  );
}
