import { useState } from "react";
import { Activity, HardDriveDownload, RotateCcw, Settings as SettingsIcon, ShieldCheck, Users } from "lucide-react";
import { cn } from "@/lib/utils";
import { useHealth } from "@/features/health/useHealth";
import { HealthBanner } from "@/features/health/HealthBanner";
import { StatusCard } from "@/features/health/StatusCard";
import { SettingsPage } from "@/features/settings/SettingsPage";
import { AccountsPage } from "@/features/accounts/AccountsPage";
import { BackupPage } from "@/features/backup/BackupPage";
import { RestorePage } from "@/features/restore/RestorePage";

type Tab = "status" | "accounts" | "backup" | "restore" | "settings";

export default function App() {
  const { report, loading, refresh, recheck } = useHealth();
  const [tab, setTab] = useState<Tab>("status");

  return (
    <div className="flex h-full flex-col">
      <HealthBanner report={report} loading={loading} onRefresh={refresh} />
      <div className="flex flex-1 overflow-hidden">
        <nav className="flex w-48 shrink-0 flex-col gap-1 border-r border-slate-800 bg-slate-900/40 p-3">
          <Brand />
          <NavItem icon={<Activity className="h-4 w-4" />} label="Status" active={tab === "status"} onClick={() => setTab("status")} />
          <NavItem icon={<Users className="h-4 w-4" />} label="Accounts" active={tab === "accounts"} onClick={() => setTab("accounts")} />
          <NavItem icon={<HardDriveDownload className="h-4 w-4" />} label="Backup" active={tab === "backup"} onClick={() => setTab("backup")} />
          <NavItem icon={<RotateCcw className="h-4 w-4" />} label="Restore" active={tab === "restore"} onClick={() => setTab("restore")} />
          <NavItem icon={<SettingsIcon className="h-4 w-4" />} label="Settings" active={tab === "settings"} onClick={() => setTab("settings")} />
          {/* Reserved for v0.1+: About */}
        </nav>
        <main className="flex-1 overflow-auto p-5">
          {tab === "status" && <StatusCard report={report} loading={loading} onRecheck={recheck} />}
          {tab === "accounts" && <AccountsPage health={report} />}
          {tab === "backup" && <BackupPage health={report} />}
          {tab === "restore" && <RestorePage health={report} />}
          {tab === "settings" && <SettingsPage onSaved={refresh} />}
        </main>
      </div>
    </div>
  );
}

function Brand() {
  return (
    <div className="mb-3 flex items-center gap-2 px-2 py-1">
      <ShieldCheck className="h-5 w-5 text-brand" />
      <span className="text-lg font-semibold tracking-tight">Aegis</span>
    </div>
  );
}

function NavItem({
  icon,
  label,
  active,
  onClick,
}: {
  icon: React.ReactNode;
  label: string;
  active: boolean;
  onClick: () => void;
}) {
  return (
    <button
      onClick={onClick}
      className={cn(
        "flex items-center gap-2 rounded-md px-3 py-2 text-sm transition-colors",
        active ? "bg-brand/15 text-brand-glow" : "text-slate-300 hover:bg-slate-800"
      )}
    >
      {icon}
      {label}
    </button>
  );
}
