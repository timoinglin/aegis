// Mirrors the Rust models in src-tauri/src/models.rs.
// Keep these two in sync — the IPC boundary is the only contract between them.

export type HealthStatus = "ok" | "warn" | "error" | "unknown";

/**
 * The single shape every health/error surface uses.
 * Golden rule: the UI shows `title` / `why` / `fix` — never a raw error.
 * The raw (redacted) detail is written to the op-log by the Rust layer instead.
 */
export interface HealthCheck {
  id: string;
  /** Group label for the Status card, e.g. "Connectivity". */
  category: string;
  status: HealthStatus;
  /** Plain-language summary of what's wrong (or "All good"). */
  title: string;
  /** Why it matters to the user. */
  why: string;
  /** Numbered, do-able steps to fix it. Empty when status is ok. */
  fix: string[];
  /** True once a real feature populates this slot; false = reserved/dormant. */
  active: boolean;
}

export interface HealthReport {
  checks: HealthCheck[];
  /** Worst status across all active checks — drives the header banner. */
  overall: HealthStatus;
  /** Unix millis of when this report was produced. */
  checkedAt: number;
}

export interface Settings {
  dbHost: string;
  dbPort: number;
  dbUser: string;
  /** Stored but never written to the op-log. */
  dbPassword: string;
  /** The repack's "_Server" folder (contains mysql\bin). */
  serverPath: string | null;
  /** Where backups are written. null = default (%APPDATA%\Aegis\backups). */
  backupDir: string | null;
  raHost: string;
  raPort: number;
  raUser: string;
  raPassword: string;
}

export interface DbBackupInfo {
  name: string;
  tables: number;
  approxRows: number;
}

export interface BackupResult {
  path: string;
  sizeBytes: number;
  /** True when the dump ended with mysqldump's completion footer. */
  completed: boolean;
  databases: DbBackupInfo[];
  durationSecs: number;
}

export interface BackupFile {
  name: string;
  path: string;
  sizeBytes: number;
  modifiedMs: number;
}

export interface RestoreResult {
  restoredFrom: string;
  safetyBackupPath: string;
  databases: DbBackupInfo[];
}
