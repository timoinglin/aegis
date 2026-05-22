import { useEffect, useState } from "react";
import { CalendarClock, CheckCircle2, AlertTriangle } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { getSettings, applySchedule, scheduleStatus } from "@/lib/ipc";
import type { ScheduleStatus, Settings } from "@/lib/types";

const inputCls =
  "rounded-md border border-slate-700 bg-slate-950 px-3 py-1.5 text-sm outline-none focus:border-brand";

const WEEKDAYS = [
  ["MON", "Monday"], ["TUE", "Tuesday"], ["WED", "Wednesday"], ["THU", "Thursday"],
  ["FRI", "Friday"], ["SAT", "Saturday"], ["SUN", "Sunday"],
];

/** Configure Windows to run a full backup automatically on a schedule. */
export function ScheduleCard() {
  const [settings, setSettings] = useState<Settings | null>(null);
  const [status, setStatus] = useState<ScheduleStatus | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [saved, setSaved] = useState(false);

  useEffect(() => {
    void getSettings().then(setSettings);
    void scheduleStatus().then(setStatus);
  }, []);

  if (!settings) return null;

  const set = <K extends keyof Settings>(key: K, value: Settings[K]) => {
    setSettings({ ...settings, [key]: value });
    setSaved(false);
  };

  const save = async () => {
    setBusy(true);
    setError(null);
    setSaved(false);
    try {
      setStatus(await applySchedule(settings));
      setSaved(true);
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };

  const enabled = settings.backupScheduleEnabled;

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <CalendarClock className="h-4 w-4 text-brand" />
          Automatic backups
        </CardTitle>
      </CardHeader>
      <CardContent className="flex flex-col gap-3">
        <label className="flex items-center gap-2 text-sm">
          <input
            type="checkbox"
            className="h-4 w-4 accent-brand"
            checked={enabled}
            onChange={(e) => set("backupScheduleEnabled", e.target.checked)}
          />
          Back up automatically on a schedule
        </label>

        {enabled && (
          <div className="flex flex-col gap-3 rounded-lg bg-slate-800/30 p-3">
            <div className="flex flex-wrap items-end gap-3">
              <Labeled label="How often">
                <select className={inputCls} value={settings.backupFrequency} onChange={(e) => set("backupFrequency", e.target.value)}>
                  <option value="daily">Daily</option>
                  <option value="weekly">Weekly</option>
                </select>
              </Labeled>
              {settings.backupFrequency === "weekly" && (
                <Labeled label="Day">
                  <select className={inputCls} value={settings.backupWeekday} onChange={(e) => set("backupWeekday", e.target.value)}>
                    {WEEKDAYS.map(([code, name]) => (
                      <option key={code} value={code}>{name}</option>
                    ))}
                  </select>
                </Labeled>
              )}
              <Labeled label="At">
                <input type="time" className={inputCls} value={settings.backupTime} onChange={(e) => set("backupTime", e.target.value)} />
              </Labeled>
              <Labeled label="Keep last">
                <input
                  type="number"
                  min={0}
                  className={`${inputCls} w-24`}
                  value={settings.backupKeep}
                  onChange={(e) => set("backupKeep", Math.max(0, Number(e.target.value)))}
                />
              </Labeled>
            </div>
            <p className="text-xs text-slate-500">
              Older backups beyond that number are deleted automatically. Backups run when your PC is
              on; if it's off at the scheduled time, the next one runs as usual.
            </p>
          </div>
        )}

        <div className="flex items-center gap-3">
          <Button onClick={save} disabled={busy}>{busy ? "Saving…" : "Save schedule"}</Button>
          {saved && status && (
            <span className="flex items-center gap-1.5 text-sm text-emerald-300">
              <CheckCircle2 className="h-4 w-4" />
              {status.enabled
                ? `${status.summary}${status.nextRun ? ` · next: ${status.nextRun}` : ""}`
                : "Automatic backups turned off."}
            </span>
          )}
          {error && (
            <span className="flex items-center gap-1.5 text-sm text-rose-300">
              <AlertTriangle className="h-4 w-4" /> {error}
            </span>
          )}
        </div>

        {!saved && status?.enabled && (
          <p className="text-xs text-slate-400">
            Currently scheduled: {status.summary}
            {status.nextRun ? ` · next run ${status.nextRun}` : ""}.
          </p>
        )}
      </CardContent>
    </Card>
  );
}

function Labeled({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <label className="flex flex-col gap-1 text-sm">
      <span className="text-slate-300">{label}</span>
      {children}
    </label>
  );
}
