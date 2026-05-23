import { useCallback, useEffect, useState } from "react";
import { Play, Square, RotateCw, Rocket, AlertTriangle, FileText } from "lucide-react";
import { openPath } from "@tauri-apps/plugin-opener";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { getSettings, serverStatus, serverAction } from "@/lib/ipc";
import type { ServerStatus, ServiceState } from "@/lib/types";

type Row = { key: "mysql" | "authserver" | "worldserver"; label: string; note?: string };

const ROWS: Row[] = [
  { key: "mysql", label: "MySQL (database)", note: "Must be running for everything else." },
  { key: "authserver", label: "Authserver (login)" },
  { key: "worldserver", label: "Worldserver (game)", note: "Stops safely via Remote Access when possible." },
];

export function ServerPage({ onChanged }: { onChanged: () => void }) {
  const [status, setStatus] = useState<ServerStatus | null>(null);
  const [busy, setBusy] = useState<string | null>(null);
  const [message, setMessage] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [repackPath, setRepackPath] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    try {
      setStatus(await serverStatus());
    } catch {
      /* transient; next tick retries */
    }
  }, []);

  useEffect(() => {
    void refresh();
    void getSettings().then((s) => setRepackPath(s.repackPath || null));
    const t = setInterval(refresh, 4000);
    return () => clearInterval(t);
  }, [refresh]);

  const run = async (service: string, action: string, busyKey: string) => {
    setBusy(busyKey);
    setMessage(null);
    setError(null);
    try {
      const msg = await serverAction(service, action);
      setMessage(msg);
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(null);
      await refresh();
      onChanged(); // let the Health spine re-check too
    }
  };

  return (
    <div className="flex flex-col gap-4">
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Rocket className="h-4 w-4 text-brand" />
            Your server
          </CardTitle>
        </CardHeader>
        <CardContent className="flex flex-col gap-3">
          <p className="text-sm text-slate-300">
            Start, stop, and restart your server's programs — no console windows to juggle.
          </p>
          <div>
            <Button onClick={() => run("", "start_all", "start_all")} disabled={!!busy}>
              <Rocket className="h-4 w-4" />
              {busy === "start_all" ? "Starting everything…" : "Start everything"}
            </Button>
            <p className="mt-1 text-xs text-slate-500">Starts MySQL, then Authserver, then Worldserver, in order.</p>
          </div>

          {message && <pre className="whitespace-pre-wrap rounded-md bg-slate-800/40 p-2 text-xs text-emerald-300">{message}</pre>}
          {error && (
            <p className="flex items-start gap-1.5 text-sm text-rose-300">
              <AlertTriangle className="mt-0.5 h-4 w-4 shrink-0" /> {error}
            </p>
          )}
        </CardContent>
      </Card>

      <Card>
        <CardContent className="flex flex-col divide-y divide-slate-800 pt-2">
          {ROWS.map((row) => (
            <ServiceRow
              key={row.key}
              row={row}
              state={status?.[row.key]}
              busy={busy}
              onAction={(action) => run(row.key, action, `${row.key}:${action}`)}
              extra={
                row.key === "worldserver" && repackPath ? (
                  <Button
                    variant="outline"
                    size="sm"
                    title="Open worldserver.conf in Notepad"
                    onClick={() => { void openPath(`${repackPath}\\worldserver.conf`).catch(() => {}); }}
                  >
                    <FileText className="h-3.5 w-3.5" /> Open .conf
                  </Button>
                ) : null
              }
            />
          ))}
        </CardContent>
      </Card>
    </div>
  );
}

function ServiceRow({
  row,
  state,
  busy,
  onAction,
  extra,
}: {
  row: Row;
  state: ServiceState | undefined;
  busy: string | null;
  onAction: (action: string) => void;
  extra?: React.ReactNode;
}) {
  const running = state?.running ?? false;
  const launchable = state?.launchable ?? false;
  const anyBusy = !!busy;

  return (
    <div className="flex items-center gap-3 py-3">
      <span className={`h-2.5 w-2.5 shrink-0 rounded-full ${running ? "bg-emerald-400" : "bg-slate-500"}`} aria-hidden />
      <div className="flex-1">
        <div className="flex items-center gap-2">
          <span className="text-sm font-medium">{row.label}</span>
          <span className={`text-xs ${running ? "text-emerald-300" : "text-slate-400"}`}>
            · {running ? `Running${state?.pid ? ` (PID ${state.pid})` : ""}` : "Stopped"}
          </span>
        </div>
        {!launchable ? (
          <p className="mt-0.5 text-xs text-amber-300">Set its folder in Settings to control it from here.</p>
        ) : (
          row.note && <p className="mt-0.5 text-xs text-slate-500">{row.note}</p>
        )}
      </div>
      <div className="flex gap-1.5">
        {extra}
        <Button variant="outline" size="sm" disabled={anyBusy || running || !launchable} onClick={() => onAction("start")}>
          <Play className="h-3.5 w-3.5" /> Start
        </Button>
        <Button variant="outline" size="sm" disabled={anyBusy || !running} onClick={() => onAction("stop")}>
          <Square className="h-3.5 w-3.5" /> Stop
        </Button>
        <Button variant="outline" size="sm" disabled={anyBusy || !running || !launchable} onClick={() => onAction("restart")}>
          <RotateCw className="h-3.5 w-3.5" /> Restart
        </Button>
      </div>
    </div>
  );
}
