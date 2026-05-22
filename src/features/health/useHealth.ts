import { useCallback, useEffect, useState } from "react";
import { runHealthChecks, recheck as recheckOne } from "@/lib/ipc";
import type { HealthReport } from "@/lib/types";

/**
 * The Health store — the spine of the app. Every feature reads its state from
 * here; nothing shows a status that didn't come through a HealthCheck.
 */
export function useHealth() {
  const [report, setReport] = useState<HealthReport | null>(null);
  const [loading, setLoading] = useState(false);

  const refresh = useCallback(async () => {
    setLoading(true);
    try {
      setReport(await runHealthChecks());
    } finally {
      setLoading(false);
    }
  }, []);

  const recheck = useCallback(async (id: string) => {
    setLoading(true);
    try {
      setReport(await recheckOne(id));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  return { report, loading, refresh, recheck };
}
