import { useState } from "react";
import { CheckCircle2, UserPlus, ShieldHalf, KeyRound } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { createAccount, setAccountPassword, setGmLevel } from "@/lib/ipc";
import type { HealthReport } from "@/lib/types";
import { GM_LEVELS } from "./gmLevels";
import { RaSetupHelp } from "./RaSetupHelp";

const inputCls =
  "w-full rounded-md border border-slate-700 bg-slate-950 px-3 py-1.5 text-sm outline-none focus:border-brand";

type Result = { ok: boolean; message: string } | null;

/**
 * Account management via Remote Access. The worldserver does the password hashing
 * (SRP6) — Aegis never writes the auth tables directly. Because RA needs the
 * server running, the page gates itself on the live "ra" health check.
 */
export function AccountsPage({ health }: { health: HealthReport | null }) {
  const raCheck = health?.checks.find((c) => c.id === "ra");
  const raReady = raCheck?.status === "ok";

  return (
    <div className="flex flex-col gap-4">
      {!raReady && <RaSetupHelp />}

      <CreateAccountCard disabled={!raReady} />
      <SetGmLevelCard disabled={!raReady} />
      <ResetPasswordCard disabled={!raReady} />
    </div>
  );
}

function ResultLine({ result }: { result: Result }) {
  if (!result) return null;
  return (
    <p className={`mt-1 text-xs ${result.ok ? "text-emerald-300" : "text-rose-300"}`}>
      {result.ok && <CheckCircle2 className="mr-1 inline h-3.5 w-3.5" />}
      {result.message}
    </p>
  );
}

function ActionCard({
  icon,
  title,
  children,
}: {
  icon: React.ReactNode;
  title: string;
  children: React.ReactNode;
}) {
  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          {icon}
          {title}
        </CardTitle>
      </CardHeader>
      <CardContent className="flex flex-col gap-3">{children}</CardContent>
    </Card>
  );
}

function CreateAccountCard({ disabled }: { disabled: boolean }) {
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [busy, setBusy] = useState(false);
  const [result, setResult] = useState<Result>(null);

  const submit = async () => {
    setBusy(true);
    setResult(null);
    try {
      const msg = await createAccount(username, password);
      setResult({ ok: true, message: msg || `Account "${username}" created. They can log in now.` });
      setPassword("");
    } catch (e) {
      setResult({ ok: false, message: String(e) });
    } finally {
      setBusy(false);
    }
  };

  return (
    <ActionCard icon={<UserPlus className="h-4 w-4 text-brand" />} title="Create account">
      <div className="grid grid-cols-2 gap-3">
        <input className={inputCls} placeholder="Account name" value={username} onChange={(e) => setUsername(e.target.value)} />
        <input className={inputCls} type="password" placeholder="Password" value={password} onChange={(e) => setPassword(e.target.value)} />
      </div>
      <div className="flex items-center justify-between">
        <ResultLine result={result} />
        <Button onClick={submit} disabled={disabled || busy || !username || !password}>
          {busy ? "Creating…" : "Create account"}
        </Button>
      </div>
    </ActionCard>
  );
}

function SetGmLevelCard({ disabled }: { disabled: boolean }) {
  const [username, setUsername] = useState("");
  const [level, setLevel] = useState(0);
  const [busy, setBusy] = useState(false);
  const [result, setResult] = useState<Result>(null);

  const submit = async () => {
    setBusy(true);
    setResult(null);
    try {
      const msg = await setGmLevel(username, level);
      setResult({
        ok: true,
        message: msg || `Set "${username}" to GM level ${level}. They need to log out and back in for it to take effect.`,
      });
    } catch (e) {
      setResult({ ok: false, message: String(e) });
    } finally {
      setBusy(false);
    }
  };

  return (
    <ActionCard icon={<ShieldHalf className="h-4 w-4 text-brand" />} title="Set GM level">
      <div className="grid grid-cols-2 gap-3">
        <input className={inputCls} placeholder="Account name" value={username} onChange={(e) => setUsername(e.target.value)} />
        <select className={inputCls} value={level} onChange={(e) => setLevel(Number(e.target.value))}>
          {GM_LEVELS.map((l) => (
            <option key={l.value} value={l.value}>
              {l.label}
            </option>
          ))}
        </select>
      </div>
      <p className="text-xs text-slate-500">
        Applies to all realms. The change takes effect after the player logs out and back in. You can
        only assign levels below your Remote Access account's own level.
      </p>
      <div className="flex items-center justify-between">
        <ResultLine result={result} />
        <Button onClick={submit} disabled={disabled || busy || !username}>
          {busy ? "Saving…" : "Set GM level"}
        </Button>
      </div>
    </ActionCard>
  );
}

function ResetPasswordCard({ disabled }: { disabled: boolean }) {
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [busy, setBusy] = useState(false);
  const [result, setResult] = useState<Result>(null);

  const submit = async () => {
    setBusy(true);
    setResult(null);
    try {
      const msg = await setAccountPassword(username, password);
      setResult({ ok: true, message: msg || `Password reset for "${username}".` });
      setPassword("");
    } catch (e) {
      setResult({ ok: false, message: String(e) });
    } finally {
      setBusy(false);
    }
  };

  return (
    <ActionCard icon={<KeyRound className="h-4 w-4 text-brand" />} title="Reset password">
      <div className="grid grid-cols-2 gap-3">
        <input className={inputCls} placeholder="Account name" value={username} onChange={(e) => setUsername(e.target.value)} />
        <input className={inputCls} type="password" placeholder="New password" value={password} onChange={(e) => setPassword(e.target.value)} />
      </div>
      <div className="flex items-center justify-between">
        <ResultLine result={result} />
        <Button onClick={submit} disabled={disabled || busy || !username || !password}>
          {busy ? "Resetting…" : "Reset password"}
        </Button>
      </div>
    </ActionCard>
  );
}
