import { invoke } from "@tauri-apps/api/core";
import type { BackupFile, BackupResult, HealthReport, RestoreResult, Settings } from "./types";

// Thin, typed wrappers over the Rust #[tauri::command] surface.
// Components call these — never invoke() directly.

export function getSettings(): Promise<Settings> {
  return invoke<Settings>("get_settings");
}

export function saveSettings(settings: Settings): Promise<Settings> {
  return invoke<Settings>("save_settings", { settings });
}

/** Probe common install locations for the repack's "_Server" folder. */
export function autodetectServerPath(): Promise<string | null> {
  return invoke<string | null>("autodetect_server_path");
}

/** Run the full Health report (all connectivity checks + reserved slots). */
export function runHealthChecks(): Promise<HealthReport> {
  return invoke<HealthReport>("run_health_checks");
}

/** Re-run a single check by id (per-row "Re-check" button). */
export function recheck(id: string): Promise<HealthReport> {
  return invoke<HealthReport>("recheck", { id });
}

// --- Account management (via the worldserver's Remote Access console) ---
// On success these resolve with the server's own (redacted) reply; on failure
// they reject with a friendly message.

export function createAccount(username: string, password: string): Promise<string> {
  return invoke<string>("create_account", { username, password });
}

export function setGmLevel(username: string, level: number, realmId = -1): Promise<string> {
  return invoke<string>("set_gm_level", { username, level, realmId });
}

export function setAccountPassword(username: string, newPassword: string): Promise<string> {
  return invoke<string>("set_account_password", { username, newPassword });
}

// --- Backups ---

/** The resolved backup folder (custom setting, or the default). */
export function backupDir(): Promise<string> {
  return invoke<string>("backup_dir");
}

/** Run a full database backup. Resolves with size + per-DB sanity info. */
export function createBackup(): Promise<BackupResult> {
  return invoke<BackupResult>("create_backup");
}

/** Backups available to restore, newest first. */
export function listBackups(): Promise<BackupFile[]> {
  return invoke<BackupFile[]>("list_backups");
}

/**
 * Restore a backup. Destructive: the backend takes a safety backup first,
 * requires the typed confirmation word, and refuses if the server is running.
 */
export function restoreBackup(path: string, confirmation: string): Promise<RestoreResult> {
  return invoke<RestoreResult>("restore_backup", { path, confirmation });
}
