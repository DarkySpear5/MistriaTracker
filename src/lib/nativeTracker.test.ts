import { invoke } from '@tauri-apps/api/core';
import { describe, expect, it, vi } from 'vitest';
import {
  getActiveSnapshot,
  getPreferences,
  getProfileSummary,
  importSelectedSave,
  probeReadiness,
  readItemNote,
  resolveGameDirectory,
  savePreferences,
  selectProfile,
} from './nativeTracker';

vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }));

describe('native tracker bridge', () => {
  it('requests only tracker-owned profile state', async () => {
    vi.mocked(invoke).mockResolvedValueOnce({ active_profile: null, profiles: [] });

    await expect(getProfileSummary()).resolves.toEqual({ active_profile: null, profiles: [] });
    expect(invoke).toHaveBeenCalledWith('get_profile_summary');
  });

  it('selects a numeric profile through the native bridge', async () => {
    vi.mocked(invoke).mockResolvedValueOnce(undefined);

    await selectProfile('1849811906');
    expect(invoke).toHaveBeenCalledWith('select_profile', { profileId: '1849811906' });
  });

  it('reads and saves normalized local preferences', async () => {
    const preferences = {
      language_preference: 'fra' as const,
      effective_language: 'fra' as const,
      spoiler_mode: 'free' as const,
      hints_enabled: true,
    };
    vi.mocked(invoke).mockResolvedValueOnce(preferences).mockResolvedValueOnce(undefined);

    await expect(getPreferences()).resolves.toEqual(preferences);
    await savePreferences({
      language_preference: preferences.language_preference,
      spoiler_mode: preferences.spoiler_mode,
      hints_enabled: preferences.hints_enabled,
    });
    expect(invoke).toHaveBeenNthCalledWith(1, 'get_preferences');
    expect(invoke).toHaveBeenNthCalledWith(2, 'save_preferences', {
      preferences: {
        language_preference: 'fra',
        spoiler_mode: 'free',
        hints_enabled: true,
      },
    });
  });

  it('reads a note only for the requested stable item id', async () => {
    vi.mocked(invoke).mockResolvedValueOnce('Pond route');

    await expect(readItemNote('paper_pondshell')).resolves.toBe('Pond route');
    expect(invoke).toHaveBeenCalledWith('read_item_note', { itemId: 'paper_pondshell' });
  });

  it('requests only the pre-filtered active snapshot', async () => {
    vi.mocked(invoke).mockResolvedValueOnce(null);

    await expect(getActiveSnapshot()).resolves.toBeNull();
    expect(invoke).toHaveBeenCalledWith('get_active_snapshot');
  });

  it('probes setup paths without starting a session', async () => {
    const report = { catalog: { game_version: 'unversioned-assets', item_count: 120 }, catalog_approved: false, companion_log_found: false };
    vi.mocked(invoke).mockResolvedValueOnce(report);

    await expect(probeReadiness('C:/game', 'C:/mod-data')).resolves.toEqual(report);
    expect(invoke).toHaveBeenCalledWith('probe_readiness', { gameDirectory: 'C:/game', modDataDirectory: 'C:/mod-data' });
  });

  it('lets the native backend resolve the local companion-log directory', async () => {
    vi.mocked(invoke).mockResolvedValueOnce({
      catalog: { game_version: 'verified', item_count: 24 },
      catalog_approved: true,
      companion_log_found: false,
    });

    await probeReadiness('C:/game');
    const argumentsForCall = vi.mocked(invoke).mock.calls.at(-1)?.[1] as Record<string, unknown>;
    expect(argumentsForCall).toEqual({ gameDirectory: 'C:/game' });
    expect(Object.hasOwn(argumentsForCall, 'modDataDirectory')).toBe(false);
  });

  it('asks native code for a validated Fields of Mistria folder', async () => {
    vi.mocked(invoke).mockResolvedValueOnce('D:/Games/Fields of Mistria');

    await expect(resolveGameDirectory()).resolves.toBe('D:/Games/Fields of Mistria');
    expect(invoke).toHaveBeenCalledWith('resolve_game_directory');
  });

  it('passes only the user-selected save path to native import', async () => {
    vi.mocked(invoke).mockResolvedValueOnce({
      profile_id: '1849811906',
      discovered_items: 2,
      imported: true,
    });

    await expect(importSelectedSave('D:/Saves/Ari-game-1849811906-1.sav')).resolves.toEqual({
      profile_id: '1849811906',
      discovered_items: 2,
      imported: true,
    });
    expect(invoke).toHaveBeenCalledWith('import_selected_save', {
      savePath: 'D:/Saves/Ari-game-1849811906-1.sav',
    });
  });
});
