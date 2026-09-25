import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import type { LanguagePreference, SupportedLocale } from '../i18n';

export type ProfileSummary = {
  active_profile: string | null;
  profiles: string[];
};

export type NativePreferences = {
  language_preference: LanguagePreference;
  effective_language: SupportedLocale;
  spoiler_mode: 'all' | 'free';
  hints_enabled: boolean;
};

export type NativePreferencesInput = Omit<NativePreferences, 'effective_language'>;

export type ActiveSnapshot = {
  collections: { items: { completed: number; total: number } };
  items: { id: string; name: string; description: string; icon_sprite: string | null; seasons: string[]; locations: string[] }[];
  villagers: { npc_id: string; gifts: { item_id: string; item_name: string; reaction: string }[] }[];
};

export type ReadinessReport = {
  catalog: { game_version: string; item_count: number };
  catalog_approved: boolean;
  catalog_unverified?: boolean;
  companion_log_found: boolean;
};

export type ExistingImportReport = {
  status?: 'imported' | 'already_imported' | 'unsupported_version';
  profile_id: string;
  discovered_items: number;
  imported: boolean;
  game_version?: string;
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

export async function chooseAndImportSave(): Promise<ExistingImportReport | null> {
  const selection = await open({
    multiple: false,
    title: 'Choose a Fields of Mistria save file',
    filters: [{ name: 'Fields of Mistria save', extensions: ['sav'] }],
  });
  if (typeof selection !== 'string') return null;
  return importSelectedSave(selection);
}

export function importSelectedSave(savePath: string): Promise<ExistingImportReport> {
  return invoke<ExistingImportReport>('import_selected_save', { savePath });
}

export function savePreferences(preferences: NativePreferencesInput): Promise<void> {
  return invoke<void>('save_preferences', { preferences });
}

export function readItemNote(itemId: string): Promise<string | null> {
  return invoke<string | null>('read_item_note', { itemId });
}

export function saveItemNote(itemId: string, text: string): Promise<void> {
  return invoke<void>('save_item_note', { itemId, text });
}
