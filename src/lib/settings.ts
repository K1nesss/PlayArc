import { invoke } from '@tauri-apps/api/core';

export type ThemeMode = 'light' | 'dark' | 'system';
export type AppLanguage = 'zh-CN' | 'en-US';
/** Tray right-click: the PlayArc panel or the system menu. */
export type TrayStyle = 'panel' | 'native';

export type AppSettings = {
  themeMode: ThemeMode;
  language: AppLanguage;
  apiKey: string;
  steam64Id: string;
  launchOnStartup: boolean;
  minimizeToTray: boolean;
  autoStartMonitor: boolean;
  dashboardRefreshSeconds: number;
  timelinePageSize: number;
  analyticsDefaultRange: number;
  animationsEnabled: boolean;
  idleScanIntervalSeconds: number;
  activeScanIntervalSeconds: number;
  /** Windows 11 Mica behind the sidebar (§8 Appearance → Sidebar transparency). */
  sidebarTransparency: boolean;
  /** Games page opens in the dense list view instead of the cover grid. */
  compactGameList: boolean;
  /** Sync Steam artwork in the background (§8 Steam → Auto Sync). */
  steamAutoSync: boolean;
  steamSyncIntervalMinutes: number;
  trayStyle: TrayStyle;
};

/** Analysis ranges in days; 0 = all time. */
export const ANALYTICS_RANGES = [7, 30, 90, 365, 0] as const;

export const SETTINGS_STORAGE_KEY = 'pt-settings-v1';

// Keep defaults aligned with original pages behavior.
export const DEFAULT_SETTINGS: AppSettings = {
  themeMode: 'system',
  language: 'zh-CN',
  apiKey: '',
  steam64Id: '',
  launchOnStartup: false,
  minimizeToTray: true,
  autoStartMonitor: true,
  dashboardRefreshSeconds: 5,
  timelinePageSize: 80,
  analyticsDefaultRange: 7,
  animationsEnabled: true,
  idleScanIntervalSeconds: 15,
  activeScanIntervalSeconds: 3,
  sidebarTransparency: true,
  compactGameList: false,
  steamAutoSync: true,
  steamSyncIntervalMinutes: 60,
  trayStyle: 'panel',
};

let cachedSettings: AppSettings | null = null;

function normalizeRange(value: number, allow: number[], fallback: number) {
  return allow.includes(value) ? value : fallback;
}

export function mergeSettings(input: Partial<AppSettings>): AppSettings {
  const merged: AppSettings = {
    ...DEFAULT_SETTINGS,
    ...input,
  };

  merged.dashboardRefreshSeconds = normalizeRange(
    Number(merged.dashboardRefreshSeconds),
    [2, 5, 10, 30],
    DEFAULT_SETTINGS.dashboardRefreshSeconds,
  );
  merged.timelinePageSize = normalizeRange(
    Number(merged.timelinePageSize),
    [50, 80, 120, 200],
    DEFAULT_SETTINGS.timelinePageSize,
  );
  // "Today" (1) was removed from Analysis; it is Home's job now (§2).
  if (Number(merged.analyticsDefaultRange) === 1) {
    merged.analyticsDefaultRange = 7;
  }
  merged.analyticsDefaultRange = normalizeRange(
    Number(merged.analyticsDefaultRange),
    [...ANALYTICS_RANGES],
    DEFAULT_SETTINGS.analyticsDefaultRange,
  );
  merged.steamSyncIntervalMinutes = normalizeRange(
    Number(merged.steamSyncIntervalMinutes),
    [30, 60, 180, 360],
    DEFAULT_SETTINGS.steamSyncIntervalMinutes,
  );
  for (const key of ['sidebarTransparency', 'compactGameList', 'steamAutoSync'] as const) {
    if (typeof merged[key] !== 'boolean') {
      merged[key] = DEFAULT_SETTINGS[key];
    }
  }
  merged.idleScanIntervalSeconds = normalizeRange(
    Number(merged.idleScanIntervalSeconds),
    [5, 10, 15, 30],
    DEFAULT_SETTINGS.idleScanIntervalSeconds,
  );
  merged.activeScanIntervalSeconds = normalizeRange(
    Number(merged.activeScanIntervalSeconds),
    [2, 3, 5, 10],
    DEFAULT_SETTINGS.activeScanIntervalSeconds,
  );

  merged.apiKey = typeof merged.apiKey === 'string' ? merged.apiKey.trim() : '';
  merged.steam64Id =
    typeof merged.steam64Id === 'string' ? merged.steam64Id.trim() : '';

  if (!['light', 'dark', 'system'].includes(merged.themeMode)) {
    merged.themeMode = DEFAULT_SETTINGS.themeMode;
  }

  if (!['panel', 'native'].includes(merged.trayStyle)) {
    merged.trayStyle = DEFAULT_SETTINGS.trayStyle;
  }

  if (!['zh-CN', 'en-US'].includes(merged.language)) {
    merged.language = DEFAULT_SETTINGS.language;
  }

  return merged;
}

export async function loadSettingsFromStorage(): Promise<AppSettings> {
  if (cachedSettings) {
    return { ...cachedSettings };
  }

  try {
    const settings = mergeSettings(await invoke<Partial<AppSettings>>('load_app_settings'));
    const legacySettings = loadLegacySettings();
    if (legacySettings && isDefaultSettings(settings)) {
      return await saveSettingsToStorage(legacySettings);
    }

    cachedSettings = settings;
    return settings;
  } catch {
    cachedSettings = loadLegacySettings() ?? { ...DEFAULT_SETTINGS };
    return { ...cachedSettings };
  }
}

/**
 * Settings changed outside the UI (e.g. "Launch at startup" from the tray menu).
 * Updates the cache and tells an open Settings page via `pt-settings-external`.
 */
export function applyExternalSettings(input: Partial<AppSettings>) {
  cachedSettings = mergeSettings({ ...(cachedSettings ?? DEFAULT_SETTINGS), ...input });
  window.dispatchEvent(new CustomEvent('pt-settings-external', { detail: { ...cachedSettings } }));
}

export async function saveSettingsToStorage(settings: AppSettings): Promise<AppSettings> {
  const normalized = mergeSettings(settings);
  try {
    const saved = await invoke<Partial<AppSettings>>('save_app_settings', {
      settings: normalized,
    });
    localStorage.removeItem(SETTINGS_STORAGE_KEY);
    cachedSettings = mergeSettings(saved);
    return { ...cachedSettings };
  } catch {
    localStorage.setItem(SETTINGS_STORAGE_KEY, JSON.stringify(normalized));
    cachedSettings = normalized;
    return { ...cachedSettings };
  }
}

export function getCachedSettings(): AppSettings | null {
  return cachedSettings ? { ...cachedSettings } : null;
}

function loadLegacySettings(): AppSettings | null {
  try {
    const raw = localStorage.getItem(SETTINGS_STORAGE_KEY);
    if (!raw) {
      return null;
    }

    const parsed = JSON.parse(raw) as Partial<AppSettings>;
    return mergeSettings(parsed);
  } catch {
    return null;
  }
}

function isDefaultSettings(settings: AppSettings) {
  return JSON.stringify(settings) === JSON.stringify(DEFAULT_SETTINGS);
}
