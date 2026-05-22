import { useEffect, useState } from "react";
import { CheckCircle2, XCircle, ArrowRight, ArrowLeft, ServerCog } from "lucide-react";
import { Button } from "@/components/ui/button";
import { PathField } from "@/components/PathField";
import {
  autodetectClientPath,
  autodetectRepackPath,
  autodetectServerPath,
  getSettings,
  readDbConfig,
  runHealthChecks,
  saveSettings,
  testRemoteAccess,
} from "@/lib/ipc";
import type { Settings } from "@/lib/types";

const inputCls =
  "w-full rounded-md border border-slate-700 bg-slate-950 px-3 py-1.5 text-sm outline-none focus:border-brand";

const TOTAL_STEPS = 6; // welcome, repack, database, remote access, client, done

/**
 * First-run guided setup. Order matters: we find the Repack folder first because
 * its worldserver.conf holds the database connection — Aegis reads it from there,
 * then we point at the _Server tools and test, then the WoW client.
 */
export function SetupWizard({ onComplete }: { onComplete: () => void }) {
  const [step, setStep] = useState(0);
  const [draft, setDraft] = useState<Settings | null>(null);
  const [testing, setTesting] = useState(false);
  const [dbOk, setDbOk] = useState<boolean | null>(null);
  const [raTesting, setRaTesting] = useState(false);
  const [raResult, setRaResult] = useState<{ ok: boolean; msg: string } | null>(null);
  const [showRaHow, setShowRaHow] = useState(false);

  useEffect(() => {
    void getSettings().then(setDraft);
  }, []);

  const ready = draft != null;

  // Auto-detect each folder when arriving at its step (only if still empty).
  useEffect(() => {
    if (!draft) return;
    if (step === 1 && !draft.repackPath) void autodetectRepackPath(draft.serverPath).then((p) => p && set("repackPath", p));
    if (step === 2 && !draft.serverPath) void autodetectServerPath().then((p) => p && set("serverPath", p));
    if (step === 4 && !draft.clientPath) void autodetectClientPath().then((p) => p && set("clientPath", p));
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [step, ready]);

  // Whenever the repack folder is known, pull the DB connection from its config.
  useEffect(() => {
    if (draft?.repackPath) void fillFromConfig(draft.serverPath, draft.repackPath);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [draft?.repackPath]);

  if (!draft) return null;

  const set = <K extends keyof Settings>(key: K, value: Settings[K]) =>
    setDraft((d) => (d ? { ...d, [key]: value } : d));

  // Pull the real host/port/user/password straight from worldserver.conf, so a
  // server with renamed databases or a changed password just works.
  const fillFromConfig = async (serverPath: string | null, repackPath: string | null) => {
    const cfg = await readDbConfig(serverPath, repackPath);
    if (!cfg) return;
    setDraft((d) =>
      d ? { ...d, dbHost: cfg.host, dbPort: cfg.port, dbUser: cfg.user, dbPassword: cfg.password } : d
    );
  };

  const testConnection = async () => {
    setTesting(true);
    setDbOk(null);
    try {
      await saveSettings(draft);
      const report = await runHealthChecks();
      setDbOk(report.checks.find((c) => c.id === "db")?.status === "ok");
    } catch {
      setDbOk(false);
    } finally {
      setTesting(false);
    }
  };

  const testRa = async () => {
    setRaTesting(true);
    setRaResult(null);
    try {
      const msg = await testRemoteAccess(draft.raHost, draft.raPort, draft.raUser, draft.raPassword);
      setRaResult({ ok: true, msg });
    } catch (e) {
      setRaResult({ ok: false, msg: String(e) });
    } finally {
      setRaTesting(false);
    }
  };

  const next = () => setStep((s) => Math.min(s + 1, TOTAL_STEPS - 1));
  const back = () => setStep((s) => Math.max(s - 1, 0));

  const finish = async () => {
    await saveSettings({ ...draft, setupComplete: true });
    onComplete();
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-slate-950/80 backdrop-blur-sm">
      <div className="flex max-h-[90vh] w-[640px] flex-col gap-5 overflow-auto rounded-2xl border border-slate-800 bg-slate-900 p-7 shadow-2xl">
        <div className="flex items-center gap-3">
          <img src="/logo.png" alt="Aegis" className="h-12 w-12 shrink-0" />
          <div>
            <h2 className="text-lg font-semibold">Welcome to Aegis</h2>
            <p className="text-xs text-slate-400">Step {step + 1} of {TOTAL_STEPS}</p>
          </div>
        </div>

        <div className="h-1 w-full overflow-hidden rounded-full bg-slate-800">
          <div className="h-full bg-brand transition-all" style={{ width: `${((step + 1) / TOTAL_STEPS) * 100}%` }} />
        </div>

        <div className="min-h-[200px]">
          {step === 0 && (
            <div className="flex flex-col gap-3 text-sm text-slate-300">
              <p>Let's get Aegis connected to your server. It only takes a minute.</p>
              <p>We'll point Aegis at a few folders — it tries to find them automatically and checks each one is correct. You just confirm.</p>
              <p className="text-slate-400">You can change any of this later in Settings.</p>
            </div>
          )}

          {step === 1 && (
            <Field title="Your server programs" hint="The Repack folder holds authserver.exe and worldserver.exe. Aegis also reads your database settings from this folder, automatically.">
              <PathField
                label="Repack folder"
                kind="repack"
                value={draft.repackPath}
                placeholder="C:\…\MOPPREMIUM\Repack"
                onChange={(v) => set("repackPath", v)}
                onDetect={async () => set("repackPath", (await autodetectRepackPath(draft.serverPath)) ?? draft.repackPath)}
              />
            </Field>
          )}

          {step === 2 && (
            <Field title="Your database" hint="The _Server folder is the one with mysql inside it. Your login details were filled in from your server's config — just test the connection.">
              <PathField
                label="_Server folder"
                kind="server"
                value={draft.serverPath}
                placeholder="C:\…\MOPPREMIUM\Database\_Server"
                onChange={(v) => set("serverPath", v)}
                onDetect={async () => set("serverPath", (await autodetectServerPath()) ?? draft.serverPath)}
              />
              <div className="grid grid-cols-3 gap-2">
                <SmallField label="Host"><input className={inputCls} value={draft.dbHost} onChange={(e) => set("dbHost", e.target.value)} /></SmallField>
                <SmallField label="User"><input className={inputCls} value={draft.dbUser} onChange={(e) => set("dbUser", e.target.value)} /></SmallField>
                <SmallField label="Password"><input className={inputCls} type="password" value={draft.dbPassword} onChange={(e) => set("dbPassword", e.target.value)} /></SmallField>
              </div>
              <div className="flex flex-wrap items-center gap-3">
                <Button variant="outline" onClick={() => fillFromConfig(draft.serverPath, draft.repackPath)}>Re-read from config</Button>
                <Button onClick={testConnection} disabled={testing}>{testing ? "Testing…" : "Test connection"}</Button>
                {dbOk === true && <span className="flex items-center gap-1.5 text-sm text-emerald-300"><CheckCircle2 className="h-4 w-4" /> Connected!</span>}
                {dbOk === false && <span className="flex items-center gap-1.5 text-sm text-rose-300"><XCircle className="h-4 w-4" /> Couldn't connect — check the folder and password.</span>}
              </div>
            </Field>
          )}

          {step === 3 && (
            <Field title="Remote Access (for account tools)" hint="This lets Aegis create accounts, set GM levels and reset passwords. New repacks have it switched off — set it up now, or skip and do it later from the Accounts page.">
              <div className="grid grid-cols-4 gap-2">
                <SmallField label="Host"><input className={inputCls} value={draft.raHost} onChange={(e) => set("raHost", e.target.value)} /></SmallField>
                <SmallField label="Port"><input className={inputCls} type="number" value={draft.raPort} onChange={(e) => set("raPort", Number(e.target.value))} /></SmallField>
                <SmallField label="User"><input className={inputCls} value={draft.raUser} onChange={(e) => set("raUser", e.target.value)} /></SmallField>
                <SmallField label="Password"><input className={inputCls} type="password" value={draft.raPassword} onChange={(e) => set("raPassword", e.target.value)} /></SmallField>
              </div>
              <div className="flex flex-wrap items-center gap-3">
                <Button onClick={testRa} disabled={raTesting}>{raTesting ? "Testing…" : "Test Remote Access"}</Button>
                {raResult?.ok && <span className="flex items-center gap-1.5 text-sm text-emerald-300"><CheckCircle2 className="h-4 w-4" /> {raResult.msg}</span>}
                {raResult && !raResult.ok && <span className="flex items-start gap-1.5 text-sm text-amber-300"><XCircle className="mt-0.5 h-4 w-4 shrink-0" /> {raResult.msg}</span>}
              </div>
              <button type="button" onClick={() => setShowRaHow((v) => !v)} className="flex items-center gap-1.5 text-left text-xs text-brand-glow">
                <ServerCog className="h-3.5 w-3.5" /> {showRaHow ? "Hide steps" : "New repack? Here's how to switch it on"}
              </button>
              {showRaHow && (
                <ol className="list-inside list-decimal space-y-2 rounded-lg bg-slate-800/30 p-3 text-xs text-slate-300">
                  <li>Open <span className="text-slate-200">worldserver.conf</span> (in your Repack folder), find <code className="rounded bg-slate-900 px-1">Ra.Enable = 0</code> and change the 0 to a 1, then save.</li>
                  <li>Start or restart your worldserver from the <strong>Server</strong> tab.</li>
                  <li>
                    In the worldserver console window, type these two lines (choose your own name &amp; password):
                    <pre className="mt-1 select-all whitespace-pre-wrap rounded bg-slate-950 p-2 text-emerald-200">account create aegis ChangeThisPassword{"\n"}account set gmlevel aegis 9 -1</pre>
                  </li>
                  <li>Enter that same username and password above, then Test.</li>
                </ol>
              )}
            </Field>
          )}

          {step === 4 && (
            <Field title="Your WoW client" hint="The folder with Wow.exe. Aegis uses this to install handy add-ons (like the GM panel) into your game. Optional — you can skip it.">
              <PathField
                label="Client folder"
                kind="client"
                value={draft.clientPath}
                placeholder="D:\Mists of Pandaria 5-4-8"
                onChange={(v) => set("clientPath", v)}
                onDetect={async () => set("clientPath", (await autodetectClientPath()) ?? draft.clientPath)}
              />
            </Field>
          )}

          {step === 5 && (
            <div className="flex flex-col gap-3 text-sm text-slate-300">
              <div className="flex items-center gap-2 text-emerald-300"><CheckCircle2 className="h-5 w-5" /> <span className="font-medium">You're all set!</span></div>
              <p>Aegis is ready. The Status page shows everything at a glance, and you can manage your server, accounts and backups from the menu.</p>
              <SummaryLine label="Repack" value={draft.repackPath} />
              <SummaryLine label="Database" value={draft.serverPath} />
              <SummaryLine label="Remote Access" value={draft.raUser ? `${draft.raHost}:${draft.raPort} (${draft.raUser})` : "— (skipped)"} />
              <SummaryLine label="Client" value={draft.clientPath} />
            </div>
          )}
        </div>

        <div className="flex items-center justify-between">
          <Button variant="ghost" onClick={back} disabled={step === 0}><ArrowLeft className="h-4 w-4" /> Back</Button>
          {step < TOTAL_STEPS - 1 ? (
            <Button onClick={next}>Next <ArrowRight className="h-4 w-4" /></Button>
          ) : (
            <Button onClick={finish}>Finish</Button>
          )}
        </div>
      </div>
    </div>
  );
}

function Field({ title, hint, children }: { title: string; hint: string; children: React.ReactNode }) {
  return (
    <div className="flex flex-col gap-3">
      <div>
        <h3 className="text-sm font-medium text-slate-200">{title}</h3>
        <p className="mt-0.5 text-xs text-slate-400">{hint}</p>
      </div>
      {children}
    </div>
  );
}

function SmallField({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <label className="flex flex-col gap-1 text-sm">
      <span className="text-slate-300">{label}</span>
      {children}
    </label>
  );
}

function SummaryLine({ label, value }: { label: string; value: string | null }) {
  return (
    <div className="flex items-center gap-2 text-xs">
      <span className="w-20 text-slate-500">{label}</span>
      <span className="truncate text-slate-300">{value || "— (not set)"}</span>
    </div>
  );
}
