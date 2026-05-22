import { useEffect, useState } from "react";
import { getVersion } from "@tauri-apps/api/app";
import { check, type Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import { openUrl } from "@tauri-apps/plugin-opener";
import { Github, Bug, Heart, RefreshCw, Download, CheckCircle2, AlertTriangle } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";

const REPO = "https://github.com/timoinglin/aegis";
const ISSUES = `${REPO}/issues`;
const KOFI = "https://ko-fi.com/kneuma";

type UpdateState = "idle" | "checking" | "available" | "downloading" | "uptodate" | "error";

/**
 * About + the user-initiated updater. Checks GitHub Releases on demand (never
 * automatically) and, if a newer signed build exists, downloads + installs it
 * and relaunches.
 */
export function AboutPage() {
  const [version, setVersion] = useState("");
  const [state, setState] = useState<UpdateState>("idle");
  const [update, setUpdate] = useState<Update | null>(null);
  const [progress, setProgress] = useState(0);

  useEffect(() => {
    void getVersion().then(setVersion);
  }, []);

  const checkForUpdates = async () => {
    setState("checking");
    try {
      const found = await check();
      if (found) {
        setUpdate(found);
        setState("available");
      } else {
        setState("uptodate");
      }
    } catch {
      setState("error");
    }
  };

  const installUpdate = async () => {
    if (!update) return;
    setState("downloading");
    setProgress(0);
    try {
      let total = 0;
      let downloaded = 0;
      await update.downloadAndInstall((event) => {
        if (event.event === "Started") total = event.data.contentLength ?? 0;
        else if (event.event === "Progress") {
          downloaded += event.data.chunkLength;
          if (total > 0) setProgress(Math.round((downloaded / total) * 100));
        }
      });
      await relaunch();
    } catch {
      setState("error");
    }
  };

  return (
    <div className="flex flex-col gap-4">
      <Card>
        <CardContent className="flex items-center gap-4 py-5">
          <img src="/logo.png" width={72} height={72} alt="Aegis" />
          <div>
            <h2 className="text-xl font-semibold">Aegis</h2>
            <p className="text-sm text-slate-400">Version {version || "…"}</p>
            <p className="mt-1 max-w-md text-sm text-slate-300">
              One-click database &amp; server management for your EmuCoach Mists of Pandaria server.
            </p>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <RefreshCw className="h-4 w-4 text-brand" />
            Updates
          </CardTitle>
        </CardHeader>
        <CardContent className="flex flex-col gap-3">
          {state === "available" && update ? (
            <>
              <p className="text-sm text-emerald-300">
                Version {update.version} is available (you have {version}).
              </p>
              {update.body && (
                <pre className="max-h-32 overflow-auto whitespace-pre-wrap rounded-md bg-slate-800/40 p-2 text-xs text-slate-300">
                  {update.body}
                </pre>
              )}
              <div>
                <Button onClick={installUpdate}>
                  <Download className="h-4 w-4" />
                  Update &amp; restart
                </Button>
              </div>
            </>
          ) : state === "downloading" ? (
            <div>
              <p className="text-sm text-slate-300">Downloading update… {progress}%</p>
              <div className="mt-2 h-2 w-full overflow-hidden rounded-full bg-slate-800">
                <div className="h-full bg-brand transition-all" style={{ width: `${progress}%` }} />
              </div>
              <p className="mt-2 text-xs text-slate-400">Aegis will restart when it's done.</p>
            </div>
          ) : (
            <div className="flex items-center gap-3">
              <Button onClick={checkForUpdates} disabled={state === "checking"}>
                <RefreshCw className={`h-4 w-4 ${state === "checking" ? "animate-spin" : ""}`} />
                {state === "checking" ? "Checking…" : "Check for updates"}
              </Button>
              {state === "uptodate" && (
                <span className="flex items-center gap-1.5 text-sm text-emerald-300">
                  <CheckCircle2 className="h-4 w-4" /> You're on the latest version.
                </span>
              )}
              {state === "error" && (
                <span className="flex items-center gap-1.5 text-sm text-amber-300">
                  <AlertTriangle className="h-4 w-4" /> Couldn't check right now — maybe there's no
                  release yet, or you're offline.
                </span>
              )}
            </div>
          )}
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Links</CardTitle>
        </CardHeader>
        <CardContent className="flex flex-col gap-2">
          <LinkRow icon={<Github className="h-4 w-4" />} label="Project on GitHub" onClick={() => openUrl(REPO)} />
          <LinkRow icon={<Bug className="h-4 w-4" />} label="Report a problem / get help" onClick={() => openUrl(ISSUES)} />
          <LinkRow icon={<Heart className="h-4 w-4 text-rose-400" />} label="Support Aegis on Ko-fi" onClick={() => openUrl(KOFI)} />
        </CardContent>
      </Card>

      <p className="px-1 text-xs text-slate-500">
        Free &amp; open-source under the MIT license. Made by Kneuma. Aegis is an independent
        community tool and isn't affiliated with EmuCoach.
      </p>
    </div>
  );
}

function LinkRow({
  icon,
  label,
  onClick,
}: {
  icon: React.ReactNode;
  label: string;
  onClick: () => void;
}) {
  return (
    <button
      onClick={onClick}
      className="flex items-center gap-2 rounded-md px-2 py-1.5 text-left text-sm text-slate-200 transition-colors hover:bg-slate-800"
    >
      {icon}
      {label}
    </button>
  );
}
