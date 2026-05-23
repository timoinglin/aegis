import { invoke } from "@tauri-apps/api/core";
import type {
  AddonInfo,
  BackupFile,
  BackupResult,
  CharacterInfo,
  DbConnInfo,
  DbSize,
  HealthReport,
  MaintenanceResult,
  RestoreResult,
  ScheduleStatus,
  ServerStatus,
  Settings,
} from "./types";

// Thin, typed wrappers over the Rust #[tauri::command] surface.
// Components call these — never invoke() directly.

export function getSettings(): Promise<Settings> {
  return invoke<Settings>("get_settings");
}

export function saveSettings(settings: Settings): Promise<Settings> {
  return invoke<Settings>("save_settings", { settings });
}

/** Probe common install locations for the repack's "_Server" folder. Pass the
 * known Repack folder if you have it — Aegis derives the sibling _Server from it. */
export function autodetectServerPath(repackPath: string | null = null): Promise<string | null> {
  return invoke<string | null>("autodetect_server_path", { repackPath });
}

/** Guess the "Repack" folder (optionally using the known _Server path). */
export function autodetectRepackPath(serverPath: string | null): Promise<string | null> {
  return invoke<string | null>("autodetect_repack_path", { serverPath });
}

/** Guess the WoW client folder. */
export function autodetectClientPath(): Promise<string | null> {
  return invoke<string | null>("autodetect_client_path");
}

/** Verify a folder is what we expect. kind: "server" | "repack" | "client". */
export function validatePath(kind: "server" | "repack" | "client", path: string): Promise<boolean> {
  return invoke<boolean>("validate_path", { kind, path });
}

/** Actually log in to Remote Access and run a harmless command. */
export function testRemoteAccess(host: string, port: number, user: string, password: string): Promise<string> {
  return invoke<string>("test_remote_access", { host, port, user, password });
}

/** Read the real DB connection (host/port/user/pass + DB names) from worldserver.conf. */
export function readDbConfig(
  serverPath: string | null,
  repackPath: string | null
): Promise<DbConnInfo | null> {
  return invoke<DbConnInfo | null>("read_db_config", { serverPath, repackPath });
}

// --- Server process control ---

export function serverStatus(): Promise<ServerStatus> {
  return invoke<ServerStatus>("server_status");
}

/** action: "start" | "stop" | "restart" (per service) or "start_all". */
export function serverAction(service: string, action: string): Promise<string> {
  return invoke<string>("server_action", { service, action });
}

/** Database upkeep. mode: "analyze" | "optimize" | "repair". */
export function dbMaintenance(mode: "analyze" | "optimize" | "repair"): Promise<MaintenanceResult> {
  return invoke<MaintenanceResult>("db_maintenance", { mode });
}

/** On-disk size + table count for each game database. */
export function dbSizes(): Promise<DbSize[]> {
  return invoke<DbSize[]>("db_sizes");
}

/** Open a local file with the OS default app (Notepad for .conf, etc.). */
export function openInDefaultApp(path: string): Promise<void> {
  return invoke<void>("open_in_default_app", { path });
}

// --- Characters (.pdump via Remote Access) ---

export function listCharacters(): Promise<CharacterInfo[]> {
  return invoke<CharacterInfo[]>("list_characters");
}

/** Back up one character (by name or GUID). */
export function backupCharacter(nameOrGuid: string): Promise<string> {
  return invoke<string>("backup_character", { nameOrGuid });
}

/** Back up every character (one RA session). */
export function backupAllCharacters(): Promise<string> {
  return invoke<string>("backup_all_characters");
}

/** Character dump files available to import. */
export function listCharacterBackups(): Promise<BackupFile[]> {
  return invoke<BackupFile[]>("list_character_backups");
}

/** Import a character dump into an account (GUID auto-assigned). newName optional. */
export function importCharacter(path: string, account: string, newName: string): Promise<string> {
  return invoke<string>("import_character", { path, account, newName });
}

// --- Add-ons ---

/** Catalog with install/update state (checks GitHub for latest versions). */
export function listAddons(): Promise<AddonInfo[]> {
  return invoke<AddonInfo[]>("list_addons");
}

/** Base64 data-URI thumbnail for a bundled add-on (null if none). */
export function addonThumbnail(id: string): Promise<string | null> {
  return invoke<string | null>("addon_thumbnail", { id });
}

/** Download + install (or update) an add-on into the client's AddOns folder. */
export function installAddon(id: string): Promise<string> {
  return invoke<string>("install_addon", { id });
}

/** Remove an installed add-on (all of its folders) from the client's AddOns. */
export function uninstallAddon(id: string): Promise<string> {
  return invoke<string>("uninstall_addon", { id });
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

/** Delete an account and ALL its characters via Remote Access. Destructive. */
export function deleteAccount(username: string): Promise<string> {
  return invoke<string>("delete_account", { username });
}

/** Direct-DB GM level override. Use only when RA refuses with "low security level". */
export function setGmLevelDirect(username: string, level: number, realmId = -1): Promise<string> {
  return invoke<string>("set_gm_level_direct", { username, level, realmId });
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

/** Back up only the registration-portal (web_*) tables. */
export function createWebBackup(): Promise<BackupResult> {
  return invoke<BackupResult>("create_web_backup");
}

/** Backups available to restore, newest first. */
export function listBackups(): Promise<BackupFile[]> {
  return invoke<BackupFile[]>("list_backups");
}

/** Current state of the automatic-backup scheduled task. */
export function scheduleStatus(): Promise<ScheduleStatus> {
  return invoke<ScheduleStatus>("schedule_status");
}

/** Save settings and make the Windows scheduled task match them. */
export function applySchedule(settings: Settings): Promise<ScheduleStatus> {
  return invoke<ScheduleStatus>("apply_schedule", { settings });
}

/**
 * Restore a backup. Destructive: the backend takes a safety backup first,
 * requires the typed confirmation word, and refuses if the server is running.
 */
export function restoreBackup(path: string, confirmation: string): Promise<RestoreResult> {
  return invoke<RestoreResult>("restore_backup", { path, confirmation });
}
