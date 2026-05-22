import { useCallback, useEffect, useState } from "react";
import { Puzzle, Download, CheckCircle2, ArrowUpCircle, AlertTriangle, FolderX } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { getSettings, listAddons, installAddon, addonThumbnail } from "@/lib/ipc";
import type { AddonInfo } from "@/lib/types";

/**
 * Add-ons: detect what's installed in the client, recommend what's missing, and
 * install/update straight from GitHub releases — no manual unzipping.
 */
export function AddonsPage() {
  const [clientSet, setClientSet] = useState<boolean | null>(null);
  const [addons, setAddons] = useState<AddonInfo[] | null>(null);
  const [busy, setBusy] = useState<string | null>(null);
  const [message, setMessage] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const refresh = useCallback(async () => {
    setAddons(null);
    setAddons(await listAddons());
  }, []);

  useEffect(() => {
    void getSettings().then((s) => setClientSet(!!s.clientPath));
    void refresh();
  }, [refresh]);

  const install = async (id: string) => {
    setBusy(id);
    setMessage(null);
    setError(null);
    try {
      setMessage(await installAddon(id));
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(null);
      await refresh();
    }
  };

  return (
    <div className="flex flex-col gap-4">
      {clientSet === false && (
        <Card className="border-amber-500/40">
          <CardContent className="flex items-start gap-3 py-3">
            <FolderX className="mt-0.5 h-5 w-5 shrink-0 text-amber-400" />
            <div>
              <p className="text-sm font-medium text-amber-300">Set your WoW client folder first</p>
              <p className="mt-0.5 text-xs text-slate-400">
                Aegis installs add-ons into your game's AddOns folder. Add the client folder in
                Settings to enable this.
              </p>
            </div>
          </CardContent>
        </Card>
      )}

      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Puzzle className="h-4 w-4 text-brand" />
            Add-ons
          </CardTitle>
        </CardHeader>
        <CardContent className="flex flex-col gap-3">
          <p className="text-sm text-slate-300">
            Handy add-ons for your server, installed straight into your game.
          </p>

          {message && (
            <p className="flex items-start gap-1.5 text-sm text-emerald-300">
              <CheckCircle2 className="mt-0.5 h-4 w-4 shrink-0" /> {message}
            </p>
          )}
          {error && (
            <p className="flex items-start gap-1.5 text-sm text-rose-300">
              <AlertTriangle className="mt-0.5 h-4 w-4 shrink-0" /> {error}
            </p>
          )}

          {addons === null ? (
            <p className="text-sm text-slate-400">Checking for add-ons…</p>
          ) : (
            <div className="flex flex-col divide-y divide-slate-800 rounded-lg border border-slate-800">
              {addons.map((a) => (
                <AddonRow key={a.id} addon={a} busy={busy === a.id} disabled={!clientSet || !!busy} onInstall={() => install(a.id)} />
              ))}
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}

function AddonRow({
  addon,
  busy,
  disabled,
  onInstall,
}: {
  addon: AddonInfo;
  busy: boolean;
  disabled: boolean;
  onInstall: () => void;
}) {
  const [thumb, setThumb] = useState<string | null>(null);
  useEffect(() => {
    if (!addon.imageUrl && addon.hasThumbnail) void addonThumbnail(addon.id).then(setThumb);
  }, [addon.id, addon.hasThumbnail, addon.imageUrl]);

  const image = addon.imageUrl ?? thumb;

  return (
    <div className={`flex items-start gap-3 p-3 ${addon.featured ? "border-l-2 border-brand bg-brand/5" : ""}`}>
      {image && (
        <img
          src={image}
          alt=""
          className={`${addon.featured ? "w-48" : "w-28"} aspect-[4/3] shrink-0 rounded-md object-cover`}
        />
      )}
      <div className="flex-1">
        <div className="flex flex-wrap items-center gap-2">
          <span className="text-sm font-medium">{addon.name}</span>
          {addon.featured && (
            <span className="rounded-full bg-brand/20 px-2 py-0.5 text-[10px] font-semibold uppercase tracking-wide text-brand-glow">
              Recommended
            </span>
          )}
          <Status addon={addon} />
        </div>
        <p className="mt-0.5 text-xs text-slate-400">{addon.description}</p>
        {!addon.installed && !addon.featured && !disabled && (
          <p className="mt-1 text-xs text-brand-glow">Recommended — you don't have this yet.</p>
        )}
      </div>
      <div className="shrink-0">
        {addon.installed && !addon.updateAvailable ? (
          <Button variant="ghost" size="sm" disabled>
            <CheckCircle2 className="h-3.5 w-3.5 text-emerald-400" /> Up to date
          </Button>
        ) : (
          <Button size="sm" disabled={disabled || busy} onClick={onInstall}>
            {addon.updateAvailable ? <ArrowUpCircle className="h-3.5 w-3.5" /> : <Download className="h-3.5 w-3.5" />}
            {busy ? "Working…" : addon.updateAvailable ? "Update" : "Install"}
          </Button>
        )}
      </div>
    </div>
  );
}

function Status({ addon }: { addon: AddonInfo }) {
  if (addon.updateAvailable) {
    return (
      <span className="text-xs text-amber-300">
        · Update available ({addon.installedVersion} → {addon.latestVersion})
      </span>
    );
  }
  if (addon.installed) {
    return <span className="text-xs text-emerald-300">· Installed (v{addon.installedVersion})</span>;
  }
  return (
    <span className="text-xs text-slate-400">
      · Not installed{addon.latestVersion ? ` (latest v${addon.latestVersion})` : ""}
    </span>
  );
}
