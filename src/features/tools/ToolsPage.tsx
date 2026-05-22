import { Github, Bot, LayoutPanelLeft, Globe, Smartphone } from "lucide-react";
import { openUrl } from "@tauri-apps/plugin-opener";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";

type Tool = {
  name: string;
  blurb: string;
  url: string;
  icon: React.ReactNode;
};

const TOOLS: Tool[] = [
  {
    name: "wow-server-mcp",
    blurb:
      "Manage and edit your server — NPCs, quests, loot, rates, accounts, database — by just asking an AI assistant. No SQL or commands needed.",
    url: "https://github.com/timoinglin/wow-server-mcp",
    icon: <Bot className="h-5 w-5 text-brand" />,
  },
  {
    name: "MoP_GM",
    blurb:
      "A clean in-game panel for the most-used GM commands (spawn, teleport, items, cheats). You can install it straight from the Add-ons tab.",
    url: "https://github.com/timoinglin/MoP_GM",
    icon: <LayoutPanelLeft className="h-5 w-5 text-brand" />,
  },
  {
    name: "wow-mop-registration",
    blurb:
      "A full website for your server: account registration, news, forum, public armory, and an admin dashboard. This is what creates the web_* tables — back them up from the Backup tab.",
    url: "https://github.com/timoinglin/wow-mop-registration",
    icon: <Globe className="h-5 w-5 text-brand" />,
  },
  {
    name: "tele-wow",
    blurb:
      "Watch and control your server from your phone, with instant Telegram alerts the moment the worldserver, authserver or MySQL goes down.",
    url: "https://github.com/timoinglin/tele-wow",
    icon: <Smartphone className="h-5 w-5 text-brand" />,
  },
];

/** Links + blurbs for the other free MoP-server tools (all on GitHub). */
export function ToolsPage() {
  return (
    <div className="flex flex-col gap-4">
      <p className="text-sm text-slate-300">
        Here are other useful tools for running your MoP server.
      </p>

      {TOOLS.map((t) => (
        <Card key={t.name}>
          <CardContent className="flex items-start gap-4 py-4">
            <div className="mt-0.5 shrink-0">{t.icon}</div>
            <div className="flex-1">
              <h3 className="text-sm font-semibold">{t.name}</h3>
              <p className="mt-1 text-sm text-slate-400">{t.blurb}</p>
            </div>
            <Button variant="outline" size="sm" className="shrink-0" onClick={() => openUrl(t.url)}>
              <Github className="h-3.5 w-3.5" /> GitHub
            </Button>
          </CardContent>
        </Card>
      ))}
    </div>
  );
}
