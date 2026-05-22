import type { HealthStatus } from "@/lib/types";

// Single source of truth for how each status looks AND reads.
// Golden rule: never color alone — every status carries a text label too.
export const STATUS_META: Record<
  HealthStatus,
  { label: string; dot: string; text: string; ring: string }
> = {
  ok: { label: "All good", dot: "bg-emerald-400", text: "text-emerald-300", ring: "ring-emerald-500/30" },
  warn: { label: "Needs attention", dot: "bg-amber-400", text: "text-amber-300", ring: "ring-amber-500/30" },
  error: { label: "Action needed", dot: "bg-rose-500", text: "text-rose-300", ring: "ring-rose-500/30" },
  unknown: { label: "Not checked yet", dot: "bg-slate-500", text: "text-slate-400", ring: "ring-slate-600/30" },
};
