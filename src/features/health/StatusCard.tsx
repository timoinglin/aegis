import { Archive, Plug, RefreshCw } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { STATUS_META } from "./status";
import type { HealthCheck, HealthReport } from "@/lib/types";

const CATEGORY_ICONS: Record<string, React.ReactNode> = {
  Connectivity: <Plug className="h-4 w-4 text-slate-400" />,
  Backups: <Archive className="h-4 w-4 text-slate-400" />,
};

/**
 * The Status / Overview surface. Lists every health check with its plain-language
 * title, why-it-matters, numbered fix steps, and a per-row Re-check button.
 * Reserved (dormant) slots render greyed until a feature activates them.
 */
export function StatusCard({
  report,
  loading,
  onRecheck,
}: {
  report: HealthReport | null;
  loading: boolean;
  onRecheck: (id: string) => void;
}) {
  if (!report) {
    return (
      <Card>
        <CardContent className="py-8 text-center text-sm text-slate-400">
          Running checks…
        </CardContent>
      </Card>
    );
  }

  const grouped = report.checks.reduce<Record<string, HealthCheck[]>>((acc, c) => {
    (acc[c.category] ??= []).push(c);
    return acc;
  }, {});

  return (
    <div className="flex flex-col gap-4">
      {Object.entries(grouped).map(([category, checks]) => (
        <Card key={category}>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              {CATEGORY_ICONS[category]}
              {category}
            </CardTitle>
          </CardHeader>
          <CardContent className="flex flex-col divide-y divide-slate-800">
            {checks.map((check) => (
              <CheckRow key={check.id} check={check} loading={loading} onRecheck={onRecheck} />
            ))}
          </CardContent>
        </Card>
      ))}
    </div>
  );
}

function CheckRow({
  check,
  loading,
  onRecheck,
}: {
  check: HealthCheck;
  loading: boolean;
  onRecheck: (id: string) => void;
}) {
  const meta = STATUS_META[check.status];
  const dormant = !check.active;

  return (
    <div className={`group flex items-start gap-3 py-3 ${dormant ? "opacity-50" : ""}`}>
      <span className={`mt-1.5 h-2.5 w-2.5 shrink-0 rounded-full ${meta.dot}`} aria-hidden />
      <div className="flex-1">
        <div className="flex items-center gap-2">
          <span className="text-sm font-medium">{check.title}</span>
          <span className={`text-xs ${meta.text}`}>· {meta.label}</span>
        </div>
        {check.why && <p className="mt-0.5 text-xs text-slate-400">{check.why}</p>}
        {check.fix.length > 0 && (
          <ol className="mt-2 list-inside list-decimal space-y-1 text-xs text-slate-300">
            {check.fix.map((step, i) => (
              <li key={i}>{step}</li>
            ))}
          </ol>
        )}
      </div>
      {!dormant && (
        <Button
          variant="ghost"
          size="icon"
          title="Re-check this"
          onClick={() => onRecheck(check.id)}
          disabled={loading}
          className="opacity-30 transition-opacity group-hover:opacity-100"
        >
          <RefreshCw className={`h-3.5 w-3.5 ${loading ? "animate-spin" : ""}`} />
        </Button>
      )}
    </div>
  );
}
