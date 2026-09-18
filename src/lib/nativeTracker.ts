import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';

export type ProfileSummary = {
  active_profile: string | null;
  profiles: string[];
};

export type NativePreferences = {
  language: 'eng' | 'fra';
  spoiler_mode: 'all' | 'free';
  hints_enabled: boolean;
  game_directory?: string | null;
};

export type ActiveSnapshot = {
  collections: { items: { completed: number; total: number } };
  items: { id: string; name: string; description: string; icon_sprite: string | null; seasons: string[]; locations: string[] }[];
  villagers: { npc_id: string; gifts: { item_id: string; item_name: string; reaction: string }[] }[];
};

export type ReadinessReport = {
  catalog: { game_version: string; item_count: number };
  catalog_approved: boolean;
  companion_log_found: boolean;
};

export type ExistingImportReport = {
  profile_id: string;
  discovered_items: number;
  imported: boolean;
};

export function isNativeTrackerRuntime(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
}

export function getProfileSummary(): Promise<ProfileSummary> {
  return invoke<ProfileSummary>('get_profile_summary');
}

export function selectProfile(profileId: string): Promise<void> {
  return invoke<void>('select_profile', { profileId });
}

export function getPreferences(): Promise<NativePreferences> {
  return invoke<NativePreferences>('get_preferences');
}

export function resolveGameDirectory(): Promise<string | null> {
  return invoke<string | null>('resolve_game_directory');
}

export async function chooseAndSaveGameDirectory(): Promise<string | null> {
  const selection = await open({
    directory: true,
    multiple: false,
    title: 'Choose Fields of Mistria folder',
  });
  if (typeof selection !== 'string') return null;
  return invoke<string>('save_game_directory', { gameDirectory: selection });
}

export function getActiveSnapshot(): Promise<ActiveSnapshot | null> {
  return invoke<ActiveSnapshot | null>('get_active_snapshot');
}

export function getVisibleItemIcon(itemId: string): Promise<string | null> {
  return invoke<string | null>('get_visible_item_icon', { itemId });
}

export function probeReadiness(gameDirectory: string, modDataDirectory?: string): Promise<ReadinessReport> {
  return invoke<ReadinessReport>(
    'probe_readiness',
    modDataDirectory === undefined ? { gameDirectory } : { gameDirectory, modDataDirectory },
  );
}

export function startLiveTracking(gameDirectory: string, modDataDirectory?: string): Promise<void> {
  return invoke<void>('start_live_tracking', modDataDirectory === undefined ? { gameDirectory } : { gameDirectory, modDataDirectory });
}

export function pollLiveTracking(): Promise<number> {
  return invoke<number>('poll_live_tracking');
}

export function reconcileActiveLiveSave(modDataDirectory: string): Promise<ExistingImportReport | null> {
  return invoke<ExistingImportReport | null>('reconcile_active_live_save', { modDataDirectory });
}

export function importLatestDesktopBackup(): Promise<ExistingImportReport> {
  return invoke<ExistingImportReport>('import_latest_desktop_backup');
}

export function savePreferences(preferences: NativePreferences): Promise<void> {
  return invoke<void>('save_preferences', { preferences });
}

export function readItemNote(itemId: string): Promise<string | null> {
  return invoke<string | null>('read_item_note', { itemId });
}

export function saveItemNote(itemId: string, text: string): Promise<void> {
  return invoke<void>('save_item_note', { itemId, text });
}
