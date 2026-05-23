import { useEffect, useState } from "react";
import { Save, FolderOpen } from "lucide-react";
import { open } from "@tauri-apps/plugin-dialog";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { PathField } from "@/components/PathField";
import {
  autodetectClientPath,
  autodetectRepackPath,
  autodetectServerPath,
  getSettings,
  saveSettings,
} from "@/lib/ipc";
import type { Settings } from "@/lib/types";

/**
 * Settings + Test Connection. Saving triggers a Health refresh so the user
 * immediately sees whether their new values made everything green.
 */
export function SettingsPage({ onSaved }: { onSaved: () => void }) {
  const [settings, setSettings] = useState<Settings | null>(null);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    void getSettings().then(setSettings);
  }, []);

  if (!settings) return <p className="text-sm text-slate-400">Loading settings…</p>;

  const set = <K extends keyof Settings>(key: K, value: Settings[K]) =>
    setSettings({ ...settings, [key]: value });

  const detect = async () => {
    setBusy(true);
    try {
      const found = await autodetectServerPath(settings.repackPath);
      if (found) set("serverPath", found);
    } finally {
      setBusy(false);
    }
  };

  const save = async () => {
    setBusy(true);
    try {
      setSettings(await saveSettings(settings));
      onSaved(); // re-runs Health
    } finally {
      setBusy(false);
    }
  };

  return (
    <div className="flex flex-col gap-4">
      <Card>
        <CardHeader>
          <CardTitle>Repack location</CardTitle>
        </CardHeader>
        <CardContent className="flex flex-col gap-3">
          <PathField
            label="Repack folder"
            kind="repack"
            value={settings.repackPath}
            placeholder="e.g. C:\…\MOPPREMIUM\Repack"
            hint="Holds authserver.exe and worldserver.exe — lets Aegis start/stop your server."
            onChange={(v) => set("repackPath", v)}
            onDetect={async () => set("repackPath", (await autodetectRepackPath(settings.serverPath)) ?? settings.repackPath)}
          />
          <PathField
            label="_Server folder"
            kind="server"
            value={settings.serverPath}
            placeholder="e.g. C:\…\MOPPREMIUM\Database\_Server"
            hint="The folder that contains mysql\bin."
            onChange={(v) => set("serverPath", v)}
            onDetect={detect}
          />
          <PathField
            label="WoW client folder"
            kind="client"
            value={settings.clientPath}
            placeholder="e.g. D:\Mists of Pandaria 5-4-8"
            hint="The folder with Wow.exe — used to install add-ons into your game."
            onChange={(v) => set("clientPath", v)}
            onDetect={async () => set("clientPath", (await autodetectClientPath()) ?? settings.clientPath)}
          />
          <Field label="Backup folder" hint="Leave empty to use the default (in your AppData folder).">
            <div className="flex gap-2">
              <input
                className={inputCls}
                value={settings.backupDir ?? ""}
                placeholder="Default: %APPDATA%\Aegis\backups"
                onChange={(e) => set("backupDir", e.target.value || null)}
              />
              <Button
                variant="outline"
                title="Pick a folder"
                onClick={async () => {
                  const picked = await open({
                    directory: true,
                    multiple: false,
                    defaultPath: settings.backupDir || undefined,
                  });
                  if (typeof picked === "string" && picked) set("backupDir", picked);
                }}
              >
                <FolderOpen className="h-4 w-4" /> Browse
              </Button>
            </div>
          </Field>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Database</CardTitle>
        </CardHeader>
        <CardContent className="grid grid-cols-2 gap-3">
          <Field label="Host">
            <input className={inputCls} value={settings.dbHost} onChange={(e) => set("dbHost", e.target.value)} />
          </Field>
          <Field label="Port">
            <input className={inputCls} type="number" value={settings.dbPort} onChange={(e) => set("dbPort", Number(e.target.value))} />
          </Field>
          <Field label="User">
            <input className={inputCls} value={settings.dbUser} onChange={(e) => set("dbUser", e.target.value)} />
          </Field>
          <Field label="Password">
            <input className={inputCls} type="password" value={settings.dbPassword} onChange={(e) => set("dbPassword", e.target.value)} />
          </Field>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Remote Access (for account tasks)</CardTitle>
        </CardHeader>
        <CardContent className="grid grid-cols-2 gap-3">
          <Field label="Host">
            <input className={inputCls} value={settings.raHost} onChange={(e) => set("raHost", e.target.value)} />
          </Field>
          <Field label="Port">
            <input className={inputCls} type="number" value={settings.raPort} onChange={(e) => set("raPort", Number(e.target.value))} />
          </Field>
          <Field label="User">
            <input className={inputCls} value={settings.raUser} onChange={(e) => set("raUser", e.target.value)} />
          </Field>
          <Field label="Password">
            <input className={inputCls} type="password" value={settings.raPassword} onChange={(e) => set("raPassword", e.target.value)} />
          </Field>
        </CardContent>
      </Card>

      <div className="flex justify-end">
        <Button onClick={save} disabled={busy}>
          <Save className="h-4 w-4" />
          Save &amp; re-check
        </Button>
      </div>
    </div>
  );
}

const inputCls =
  "w-full rounded-md border border-slate-700 bg-slate-950 px-3 py-1.5 text-sm outline-none focus:border-brand";

function Field({ label, hint, children }: { label: string; hint?: string; children: React.ReactNode }) {
  return (
    <label className="flex flex-col gap-1 text-sm">
      <span className="text-slate-300">{label}</span>
      {children}
      {hint && <span className="text-xs text-slate-500">{hint}</span>}
    </label>
  );
}
