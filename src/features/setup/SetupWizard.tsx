import { useEffect, useState } from "react";
import { ShieldCheck, FolderSearch, CheckCircle2, XCircle, ArrowRight, ArrowLeft } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  autodetectClientPath,
  autodetectRepackPath,
  autodetectServerPath,
  getSettings,
  runHealthChecks,
  saveSettings,
} from "@/lib/ipc";
import type { Settings } from "@/lib/types";

const inputCls =
  "w-full rounded-md border border-slate-700 bg-slate-950 px-3 py-1.5 text-sm outline-none focus:border-brand";

const TOTAL_STEPS = 5; // welcome, database, repack, client, done

/**
 * First-run guided setup. Walks the owner through the four things Aegis needs:
 * a working DB connection, the _Server folder, the Repack folder, and the WoW
 * client folder. Auto-detects each where it can.
 */
export function SetupWizard({ onComplete }: { onComplete: () => void }) {
  const [step, setStep] = useState(0);
  const [draft, setDraft] = useState<Settings | null>(null);
  const [testing, setTesting] = useState(false);
  const [dbOk, setDbOk] = useState<boolean | null>(null);

  useEffect(() => {
    void getSettings().then(setDraft);
  }, []);

  // Auto-detect a folder when arriving at its step (only if still empty).
  useEffect(() => {
    if (!draft) return;
    if (step === 1 && !draft.serverPath) void autodetectServerPath().then((p) => p && set("serverPath", p));
    if (step === 2 && !draft.repackPath) void autodetectRepackPath(draft.serverPath).then((p) => p && set("repackPath", p));
    if (step === 3 && !draft.clientPath) void autodetectClientPath().then((p) => p && set("clientPath", p));
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [step, draft?.serverPath]);

  if (!draft) return null;

  const set = <K extends keyof Settings>(key: K, value: Settings[K]) =>
    setDraft((d) => (d ? { ...d, [key]: value } : d));

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
          <ShieldCheck className="h-7 w-7 text-brand" />
          <div>
            <h2 className="text-lg font-semibold">Welcome to Aegis</h2>
            <p className="text-xs text-slate-400">Step {step + 1} of {TOTAL_STEPS}</p>
          </div>
        </div>

        <div className="h-1 w-full overflow-hidden rounded-full bg-slate-800">
          <div className="h-full bg-brand transition-all" style={{ width: `${((step + 1) / TOTAL_STEPS) * 100}%` }} />
        </div>

        <div className="min-h-[180px]">
          {step === 0 && (
            <div className="flex flex-col gap-3 text-sm text-slate-300">
              <p>Let's get Aegis connected to your server. It only takes a minute.</p>
              <p>We'll point Aegis at three folders and check the database connection. Aegis will try to find everything automatically — you just confirm.</p>
              <p className="text-slate-400">You can change any of this later in Settings.</p>
            </div>
          )}

          {step === 1 && (
            <Field
              title="Your database"
              hint="The _Server folder is the one with mysql inside it. Aegis uses your repack's own MySQL — the defaults below usually just work."
            >
              <PathInput label="_Server folder" value={draft.serverPath} placeholder="C:\…\MOPPREMIUM\Database\_Server" onChange={(v) => set("serverPath", v)} onDetect={async () => set("serverPath", (await autodetectServerPath()) ?? draft.serverPath)} />
              <div className="grid grid-cols-3 gap-2">
                <SmallField label="Host"><input className={inputCls} value={draft.dbHost} onChange={(e) => set("dbHost", e.target.value)} /></SmallField>
                <SmallField label="User"><input className={inputCls} value={draft.dbUser} onChange={(e) => set("dbUser", e.target.value)} /></SmallField>
                <SmallField label="Password"><input className={inputCls} type="password" value={draft.dbPassword} onChange={(e) => set("dbPassword", e.target.value)} /></SmallField>
              </div>
              <div className="flex items-center gap-3">
                <Button variant="outline" onClick={testConnection} disabled={testing}>{testing ? "Testing…" : "Test connection"}</Button>
                {dbOk === true && <span className="flex items-center gap-1.5 text-sm text-emerald-300"><CheckCircle2 className="h-4 w-4" /> Connected!</span>}
                {dbOk === false && <span className="flex items-center gap-1.5 text-sm text-rose-300"><XCircle className="h-4 w-4" /> Couldn't connect — check the folder and password.</span>}
              </div>
            </Field>
          )}

          {step === 2 && (
            <Field title="Your server programs" hint="The Repack folder holds authserver.exe and worldserver.exe. This lets Aegis start and stop your server for you.">
              <PathInput label="Repack folder" value={draft.repackPath} placeholder="C:\…\MOPPREMIUM\Repack" onChange={(v) => set("repackPath", v)} onDetect={async () => set("repackPath", (await autodetectRepackPath(draft.serverPath)) ?? draft.repackPath)} />
            </Field>
          )}

          {step === 3 && (
            <Field title="Your WoW client" hint="The folder with Wow.exe. Aegis uses this to install handy add-ons (like the GM panel) straight into your game. Optional — you can skip it.">
              <PathInput label="Client folder" value={draft.clientPath} placeholder="D:\Mists of Pandaria 5-4-8" onChange={(v) => set("clientPath", v)} onDetect={async () => set("clientPath", (await autodetectClientPath()) ?? draft.clientPath)} />
            </Field>
          )}

          {step === 4 && (
            <div className="flex flex-col gap-3 text-sm text-slate-300">
              <div className="flex items-center gap-2 text-emerald-300"><CheckCircle2 className="h-5 w-5" /> <span className="font-medium">You're all set!</span></div>
              <p>Aegis is ready. The Status page shows everything at a glance, and you can manage your server, accounts and backups from the menu.</p>
              <SummaryLine label="Database" value={draft.serverPath} />
              <SummaryLine label="Repack" value={draft.repackPath} />
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

function PathInput({ label, value, placeholder, onChange, onDetect }: { label: string; value: string | null; placeholder: string; onChange: (v: string | null) => void; onDetect: () => void }) {
  return (
    <label className="flex flex-col gap-1 text-sm">
      <span className="text-slate-300">{label}</span>
      <div className="flex gap-2">
        <input className={inputCls} value={value ?? ""} placeholder={placeholder} onChange={(e) => onChange(e.target.value || null)} />
        <Button variant="outline" onClick={onDetect}><FolderSearch className="h-4 w-4" /> Detect</Button>
      </div>
    </label>
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
