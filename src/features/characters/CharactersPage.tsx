import { useCallback, useEffect, useState } from "react";
import { Search, Download, Upload, CheckCircle2, AlertTriangle, Users } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import {
  listCharacters,
  backupCharacter,
  backupAllCharacters,
  listCharacterBackups,
  importCharacter,
} from "@/lib/ipc";
import { RaSetupHelp } from "@/features/accounts/RaSetupHelp";
import type { BackupFile, CharacterInfo, HealthReport } from "@/lib/types";

const inputCls =
  "rounded-md border border-slate-700 bg-slate-950 px-3 py-1.5 text-sm outline-none focus:border-brand";

export function CharactersPage({ health }: { health: HealthReport | null }) {
  const raReady = health?.checks.find((c) => c.id === "ra")?.status === "ok";

  if (!raReady) {
    return (
      <div className="flex flex-col gap-4">
        <p className="text-sm text-slate-300">
          Character backup and import use Remote Access (the server does the safe GUID handling).
          Set it up first:
        </p>
        <RaSetupHelp />
      </div>
    );
  }

  return <CharactersInner />;
}

function CharactersInner() {
  const [chars, setChars] = useState<CharacterInfo[] | null>(null);
  const [search, setSearch] = useState("");
  const [busy, setBusy] = useState<string | null>(null);
  const [msg, setMsg] = useState<string | null>(null);
  const [err, setErr] = useState<string | null>(null);

  const loadChars = useCallback(async () => setChars(await listCharacters()), []);
  useEffect(() => {
    void loadChars();
  }, [loadChars]);

  const backupOne = async (c: CharacterInfo) => {
    setBusy(`c${c.guid}`);
    setMsg(null);
    setErr(null);
    try {
      setMsg(await backupCharacter(c.name));
    } catch (e) {
      setErr(String(e));
    } finally {
      setBusy(null);
    }
  };

  const backupAll = async () => {
    setBusy("all");
    setMsg(null);
    setErr(null);
    try {
      setMsg(await backupAllCharacters());
    } catch (e) {
      setErr(String(e));
    } finally {
      setBusy(null);
    }
  };

  const filtered = (chars ?? []).filter((c) =>
    c.name.toLowerCase().includes(search.toLowerCase())
  );

  return (
    <div className="flex flex-col gap-4">
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Users className="h-4 w-4 text-brand" />
            Back up characters
          </CardTitle>
        </CardHeader>
        <CardContent className="flex flex-col gap-3">
          <p className="text-sm text-slate-300">
            Save individual characters (or all of them) to files you can re-import onto any account
            later — the server assigns a fresh character ID automatically on import.
          </p>

          <div className="flex items-center gap-3">
            <Button onClick={backupAll} disabled={!!busy}>
              <Download className="h-4 w-4" />
              {busy === "all" ? "Backing up all… (may take a few minutes)" : "Back up all characters"}
            </Button>
            <span className="text-xs text-slate-500">{chars ? `${chars.length} characters` : "Loading…"}</span>
          </div>

          {msg && (
            <p className="flex items-start gap-1.5 text-sm text-emerald-300">
              <CheckCircle2 className="mt-0.5 h-4 w-4 shrink-0" /> {msg}
            </p>
          )}
          {err && (
            <p className="flex items-start gap-1.5 text-sm text-rose-300">
              <AlertTriangle className="mt-0.5 h-4 w-4 shrink-0" /> {err}
            </p>
          )}

          <div className="flex items-center gap-2 rounded-md border border-slate-700 bg-slate-950 px-2">
            <Search className="h-4 w-4 text-slate-500" />
            <input
              className="flex-1 bg-transparent py-1.5 text-sm outline-none"
              placeholder="Search characters by name…"
              value={search}
              onChange={(e) => setSearch(e.target.value)}
            />
          </div>

          <div className="max-h-72 overflow-auto rounded-lg border border-slate-800">
            {chars === null ? (
              <p className="p-3 text-sm text-slate-400">Loading characters…</p>
            ) : filtered.length === 0 ? (
              <p className="p-3 text-sm text-slate-400">No characters match.</p>
            ) : (
              filtered.slice(0, 200).map((c) => (
                <div key={c.guid} className="flex items-center gap-3 border-b border-slate-800/60 px-3 py-2 text-sm last:border-0">
                  <span className="flex-1 truncate">{c.name}</span>
                  <span className="text-xs text-slate-400">Lvl {c.level} {c.className}</span>
                  <span className="text-xs text-slate-600">acct {c.account}</span>
                  <Button variant="outline" size="sm" disabled={!!busy} onClick={() => backupOne(c)}>
                    {busy === `c${c.guid}` ? "…" : "Back up"}
                  </Button>
                </div>
              ))
            )}
          </div>
          {filtered.length > 200 && (
            <p className="text-xs text-slate-500">Showing the first 200 — search to narrow it down.</p>
          )}
        </CardContent>
      </Card>

      <ImportCard />
    </div>
  );
}

function ImportCard() {
  const [files, setFiles] = useState<BackupFile[] | null>(null);
  const [selected, setSelected] = useState<string | null>(null);
  const [account, setAccount] = useState("");
  const [newName, setNewName] = useState("");
  const [busy, setBusy] = useState(false);
  const [msg, setMsg] = useState<string | null>(null);
  const [err, setErr] = useState<string | null>(null);

  useEffect(() => {
    void listCharacterBackups().then(setFiles);
  }, []);

  const doImport = async () => {
    if (!selected) return;
    setBusy(true);
    setMsg(null);
    setErr(null);
    try {
      setMsg(await importCharacter(selected, account, newName));
      setNewName("");
    } catch (e) {
      setErr(String(e));
    } finally {
      setBusy(false);
    }
  };

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <Upload className="h-4 w-4 text-brand" />
          Import a character
        </CardTitle>
      </CardHeader>
      <CardContent className="flex flex-col gap-3">
        <p className="text-sm text-slate-300">
          Load a character backup onto any account. Aegis lets the server pick a free character ID,
          so it won't clash with existing characters.
        </p>

        {files && files.length === 0 ? (
          <p className="text-sm text-slate-400">No character backups yet — make one above first.</p>
        ) : (
          <div className="max-h-44 overflow-auto rounded-lg border border-slate-800">
            {(files ?? []).map((f) => (
              <label
                key={f.path}
                className={`flex cursor-pointer items-center gap-3 border-b border-slate-800/60 px-3 py-2 text-sm last:border-0 ${
                  selected === f.path ? "bg-brand/10" : "hover:bg-slate-800/50"
                }`}
              >
                <input type="radio" name="charfile" className="accent-brand" checked={selected === f.path} onChange={() => setSelected(f.path)} />
                <span className="flex-1 truncate">{f.name}</span>
                <span className="text-xs text-slate-500">{new Date(f.modifiedMs).toLocaleString()}</span>
              </label>
            ))}
          </div>
        )}

        <div className="flex flex-wrap items-end gap-3">
          <label className="flex flex-col gap-1 text-sm">
            <span className="text-slate-300">Into account</span>
            <input className={inputCls} placeholder="account name or id" value={account} onChange={(e) => setAccount(e.target.value)} />
          </label>
          <label className="flex flex-col gap-1 text-sm">
            <span className="text-slate-300">New name (optional)</span>
            <input className={inputCls} placeholder="keep original" value={newName} onChange={(e) => setNewName(e.target.value)} />
          </label>
          <Button onClick={doImport} disabled={busy || !selected || !account.trim()}>
            {busy ? "Importing…" : "Import character"}
          </Button>
        </div>

        {msg && (
          <p className="flex items-start gap-1.5 text-sm text-emerald-300">
            <CheckCircle2 className="mt-0.5 h-4 w-4 shrink-0" /> {msg}
          </p>
        )}
        {err && (
          <p className="flex items-start gap-1.5 text-sm text-rose-300">
            <AlertTriangle className="mt-0.5 h-4 w-4 shrink-0" /> {err}
          </p>
        )}
      </CardContent>
    </Card>
  );
}
