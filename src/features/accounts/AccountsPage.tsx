import { useState } from "react";
import { CheckCircle2, UserPlus, ShieldHalf, KeyRound, Database, UserX, AlertTriangle } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { createAccount, deleteAccount, setAccountPassword, setGmLevel, setGmLevelDirect } from "@/lib/ipc";
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
      <DeleteAccountCard disabled={!raReady} />
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
  // RA refused with "low security level" → show the direct-DB override path.
  const [showOverride, setShowOverride] = useState(false);
  const [confirmOverride, setConfirmOverride] = useState(false);

  const submit = async () => {
    setBusy(true);
    setResult(null);
    setShowOverride(false);
    try {
      const msg = await setGmLevel(username, level);
      setResult({
        ok: true,
        message: msg || `Set "${username}" to GM level ${level}. They need to log out and back in for it to take effect.`,
      });
    } catch (e) {
      const text = String(e);
      setResult({ ok: false, message: text });
      // The RA refusal message — let the user override via direct SQL.
      if (/Remote Access account can only assign/i.test(text) || /low security/i.test(text)) {
        setShowOverride(true);
      }
    } finally {
      setBusy(false);
    }
  };

  const submitDirect = async () => {
    setBusy(true);
    try {
      const msg = await setGmLevelDirect(username, level);
      setResult({ ok: true, message: msg });
      setShowOverride(false);
      setConfirmOverride(false);
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

      {showOverride && (
        <div className="mt-2 rounded-md border border-amber-500/40 bg-amber-500/5 p-3 text-xs">
          <p className="flex items-center gap-1.5 font-medium text-amber-200">
            <Database className="h-3.5 w-3.5" /> Override via direct SQL?
          </p>
          <p className="mt-1 text-slate-300">
            Remote Access refused because it can't grant levels at or above its own. Aegis can write the change
            straight to the <code className="rounded bg-slate-900 px-1">auth.account_access</code> table
            instead — this bypasses RA's safety check, so only use it if you really mean to.
          </p>
          <label className="mt-2 flex items-center gap-2 text-slate-300">
            <input
              type="checkbox"
              checked={confirmOverride}
              onChange={(e) => setConfirmOverride(e.target.checked)}
            />
            I understand and want to apply this directly in the database.
          </label>
          <div className="mt-2 flex justify-end gap-2">
            <Button variant="outline" size="sm" onClick={() => { setShowOverride(false); setConfirmOverride(false); }}>
              Cancel
            </Button>
            <Button size="sm" disabled={busy || !confirmOverride} onClick={submitDirect}>
              {busy ? "Applying…" : "Apply override"}
            </Button>
          </div>
        </div>
      )}
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

/**
 * Permanent account delete via RA. Wraps `.account delete` and gates it on a
 * typed-confirmation so a slip can't wipe an account by accident. The server
 * also removes the account's characters automatically.
 */
function DeleteAccountCard({ disabled }: { disabled: boolean }) {
  const [username, setUsername] = useState("");
  const [confirm, setConfirm] = useState("");
  const [busy, setBusy] = useState(false);
  const [result, setResult] = useState<Result>(null);

  // Re-typing the account name is the safety gate.
  const matches = username.trim().length > 0 && confirm.trim() === username.trim();

  const submit = async () => {
    setBusy(true);
    setResult(null);
    try {
      const msg = await deleteAccount(username.trim());
      setResult({
        ok: true,
        message: msg || `Account "${username.trim()}" deleted (along with its characters).`,
      });
      setUsername("");
      setConfirm("");
    } catch (e) {
      setResult({ ok: false, message: String(e) });
    } finally {
      setBusy(false);
    }
  };

  return (
    <ActionCard icon={<UserX className="h-4 w-4 text-rose-400" />} title="Delete account">
      <div className="flex items-start gap-2 rounded-md border border-rose-500/30 bg-rose-500/5 p-2 text-xs text-rose-200">
        <AlertTriangle className="mt-0.5 h-3.5 w-3.5 shrink-0" />
        <p>
          This <strong>permanently</strong> removes the account <em>and every character on it</em>.
          There's no undo. If you only need to lock someone out, reset their password instead.
        </p>
      </div>
      <div className="grid grid-cols-2 gap-3">
        <input
          className={inputCls}
          placeholder="Account name"
          value={username}
          onChange={(e) => setUsername(e.target.value)}
        />
        <input
          className={inputCls}
          placeholder="Re-type the name to confirm"
          value={confirm}
          onChange={(e) => setConfirm(e.target.value)}
        />
      </div>
      <div className="flex items-center justify-between">
        <ResultLine result={result} />
        <Button
          onClick={submit}
          disabled={disabled || busy || !matches}
          className="bg-rose-600 text-white hover:bg-rose-500 disabled:bg-rose-900/40 disabled:text-rose-300/60"
        >
          {busy ? "Deleting…" : "Delete account"}
        </Button>
      </div>
    </ActionCard>
  );
}
