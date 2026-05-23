import { RefreshCw } from "lucide-react";
import { Button } from "@/components/ui/button";
import { STATUS_META } from "./status";
import type { HealthReport } from "@/lib/types";

/**
 * Persistent header banner. Summarizes the worst current status at a glance,
 * always with color AND text. Sits above every page.
 */
export function HealthBanner({
  report,
  loading,
  onRefresh,
}: {
  report: HealthReport | null;
  loading: boolean;
  onRefresh: () => void;
}) {
  const overall = report?.overall ?? "unknown";
  const meta = STATUS_META[overall];
  const issues = report?.checks.filter((c) => c.active && c.status !== "ok").length ?? 0;

  return (
    <header className="flex items-center justify-between border-b border-slate-800 bg-slate-900/80 px-5 py-2">
      <div className="flex items-center gap-3">
        <span className={`h-2.5 w-2.5 rounded-full ${meta.dot}`} aria-hidden />
        <span className={`text-sm font-medium ${meta.text}`}>
          {overall === "ok"
            ? "Everything looks healthy"
            : issues > 0
              ? `${issues} thing${issues === 1 ? "" : "s"} need${issues === 1 ? "s" : ""} your attention`
              : meta.label}
        </span>
      </div>
      <Button variant="outline" size="sm" onClick={onRefresh} disabled={loading}>
        <RefreshCw className={`h-3.5 w-3.5 ${loading ? "animate-spin" : ""}`} />
        Re-check
      </Button>
    </header>
  );
}
