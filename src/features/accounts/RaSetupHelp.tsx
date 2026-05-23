import { useEffect, useState } from "react";
import { ServerCog, Copy, Check, FileText } from "lucide-react";
import { openPath } from "@tauri-apps/plugin-opener";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { getSettings } from "@/lib/ipc";

/**
 * One-time guide to enable Remote Access on a fresh repack. Account tools need
 * RA, and you can't create the RA account *through* RA — so this walks the user
 * through the worldserver console, which can grant GM 9 (RA can't grant its own level).
 */
export function RaSetupHelp() {
  const [confPath, setConfPath] = useState<string>("your Repack folder");
  const [havePath, setHavePath] = useState(false);

  useEffect(() => {
    void getSettings().then((s) => {
      if (s.repackPath) {
        setConfPath(`${s.repackPath}\\worldserver.conf`);
        setHavePath(true);
      }
    });
  }, []);

  const openConf = async () => {
    try {
      await openPath(confPath);
    } catch {
      /* the user can still copy the path */
    }
  };

  return (
    <Card className="border-amber-500/40">
      <CardHeader>
        <CardTitle className="flex items-center gap-2 text-amber-200">
          <ServerCog className="h-4 w-4" />
          Turn on Remote Access (one-time setup)
        </CardTitle>
      </CardHeader>
      <CardContent className="flex flex-col gap-4 text-sm text-slate-300">
        <p>
          Account tools talk to your server through <strong>Remote Access</strong>. Fresh repacks
          have it switched off, so let's turn it on. You only do this once.
        </p>

        <Step n={1} title="Switch Remote Access on">
          <p>
            Open this file in Notepad:
          </p>
          <CodeLine text={confPath} />
          {havePath && (
            <Button variant="outline" size="sm" className="mt-2" onClick={openConf}>
              <FileText className="h-3.5 w-3.5" /> Open worldserver.conf
            </Button>
          )}
          <p className="mt-2">
            Find the line <code className="rounded bg-slate-800 px-1">Ra.Enable = 0</code> and change
            the <strong>0</strong> to a <strong>1</strong>, then save.
          </p>
        </Step>

        <Step n={2} title="Start your server">
          <p>
            Open the <strong>Server</strong> tab and start (or restart) your worldserver so the change
            takes effect.
          </p>
        </Step>

        <Step n={3} title="Make a GM account for Aegis">
          <p>
            In the worldserver window (the black console), type these two lines, pressing{" "}
            <strong>Enter</strong> after each. Pick your own name and password:
          </p>
          <CodeBlock
            lines={["account create aegis ChangeThisPassword", "account set gmlevel aegis 9 -1"]}
          />
          <p className="mt-1 text-xs text-slate-400">
            <code className="rounded bg-slate-800 px-1">9</code> is full admin — and it only works
            from this console window, which is exactly why we do it here.
          </p>
        </Step>

        <Step n={4} title="Tell Aegis the login">
          <p>
            Go to <strong>Settings → Remote Access</strong>, enter that same username and password,
            save, then come back here and click <strong>Re-check</strong> at the top.
          </p>
        </Step>
      </CardContent>
    </Card>
  );
}

function Step({ n, title, children }: { n: number; title: string; children: React.ReactNode }) {
  return (
    <div className="flex gap-3">
      <span className="flex h-6 w-6 shrink-0 items-center justify-center rounded-full bg-brand/20 text-xs font-semibold text-brand-glow">
        {n}
      </span>
      <div className="flex-1">
        <p className="font-medium text-slate-200">{title}</p>
        <div className="mt-1 text-slate-300">{children}</div>
      </div>
    </div>
  );
}

function CopyButton({ text }: { text: string }) {
  const [copied, setCopied] = useState(false);
  const copy = async () => {
    try {
      await navigator.clipboard.writeText(text);
      setCopied(true);
      setTimeout(() => setCopied(false), 1500);
    } catch {
      /* selection still works as a fallback */
    }
  };
  return (
    <Button variant="ghost" size="icon" onClick={copy} title="Copy">
      {copied ? <Check className="h-3.5 w-3.5 text-emerald-400" /> : <Copy className="h-3.5 w-3.5" />}
    </Button>
  );
}

function CodeLine({ text }: { text: string }) {
  return (
    <div className="mt-1 flex items-center gap-2 rounded-md bg-slate-950 px-2 py-1">
      <code className="flex-1 select-all break-all text-xs text-slate-200">{text}</code>
      <CopyButton text={text} />
    </div>
  );
}

function CodeBlock({ lines }: { lines: string[] }) {
  return (
    <div className="mt-2 flex items-start gap-2 rounded-md bg-slate-950 px-3 py-2">
      <pre className="flex-1 select-all whitespace-pre-wrap text-xs text-emerald-200">{lines.join("\n")}</pre>
      <CopyButton text={lines.join("\n")} />
    </div>
  );
}
