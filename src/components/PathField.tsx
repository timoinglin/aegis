import { useEffect, useState } from "react";
import { FolderSearch, CheckCircle2, AlertTriangle } from "lucide-react";
import { Button } from "@/components/ui/button";
import { validatePath } from "@/lib/ipc";

const inputCls =
  "w-full rounded-md border border-slate-700 bg-slate-950 px-3 py-1.5 text-sm outline-none focus:border-brand";

type Kind = "server" | "repack" | "client";

const OK_MSG: Record<Kind, string> = {
  server: "Looks right — found your database tools.",
  repack: "Looks right — found your server programs.",
  client: "Looks right — found your game.",
};
const BAD_MSG: Record<Kind, string> = {
  server: "This folder doesn't have the database tools — pick the _Server folder.",
  repack: "This folder doesn't have worldserver.exe — pick your Repack folder.",
  client: "This folder doesn't look like a WoW client — pick the one with Interface\\AddOns (or Wow.exe / Wow_64.exe).",
};

/**
 * A folder field that verifies, as you type, that it's really the right folder
 * (✓/✗ + plain-language hint). Shared by the setup wizard and Settings.
 */
export function PathField({
  label,
  kind,
  value,
  placeholder,
  hint,
  onChange,
  onDetect,
}: {
  label: string;
  kind: Kind;
  value: string | null;
  placeholder?: string;
  hint?: string;
  onChange: (v: string | null) => void;
  onDetect?: () => void;
}) {
  const [ok, setOk] = useState<boolean | null>(null);

  // Re-validate shortly after the value settles (cheap fs check on the backend).
  useEffect(() => {
    if (!value || !value.trim()) {
      setOk(null);
      return;
    }
    const t = setTimeout(() => {
      void validatePath(kind, value).then(setOk);
    }, 300);
    return () => clearTimeout(t);
  }, [value, kind]);

  return (
    <label className="flex flex-col gap-1 text-sm">
      <span className="text-slate-300">{label}</span>
      <div className="flex gap-2">
        <input
          className={inputCls}
          value={value ?? ""}
          placeholder={placeholder}
          onChange={(e) => onChange(e.target.value || null)}
        />
        {onDetect && (
          <Button variant="outline" onClick={onDetect}>
            <FolderSearch className="h-4 w-4" /> Detect
          </Button>
        )}
      </div>
      {ok === true && (
        <span className="flex items-center gap-1.5 text-xs text-emerald-300">
          <CheckCircle2 className="h-3.5 w-3.5" /> {OK_MSG[kind]}
        </span>
      )}
      {ok === false && (
        <span className="flex items-center gap-1.5 text-xs text-amber-300">
          <AlertTriangle className="h-3.5 w-3.5" /> {BAD_MSG[kind]}
        </span>
      )}
      {ok === null && hint && <span className="text-xs text-slate-500">{hint}</span>}
    </label>
  );
}
