/**
 * Steam sync status for the sidebar footer (DOIT.md §11).
 *
 * `trackSteamSync` wraps every Steam sync call in `$lib/api`, so the footer
 * reflects syncs started from any page. The last outcome is persisted by the
 * backend (`steam_sync_status`); localStorage is only a fallback for the first
 * paint and for running outside Tauri.
 */

import { invoke } from '@tauri-apps/api/core';

export type SyncState = 'idle' | 'syncing' | 'error';

const LAST_SYNC_KEY = 'velaris_last_steam_sync_v1';

function readLastSync(): number | null {
	try {
		const value = Number(localStorage.getItem(LAST_SYNC_KEY));
		return Number.isFinite(value) && value > 0 ? value : null;
	} catch {
		return null;
	}
}

export const syncStatus = $state({
	state: 'idle' as SyncState,
	lastSyncedAt: null as number | null,
	error: '' as string,
	/** Whether a Steam API key + Steam ID are configured. Set by the layout. */
	configured: false
});

let active = 0;

type BackendStatus = {
	last_synced_at: number | null;
	last_error: string | null;
	last_error_at: number | null;
	running: boolean;
};

export function initSyncStatus(configured: boolean) {
	syncStatus.lastSyncedAt = readLastSync();
	syncStatus.configured = configured;

	// Backend is the source of truth once it answers.
	invoke<BackendStatus>('steam_sync_status')
		.then((status) => {
			if (status.last_synced_at) syncStatus.lastSyncedAt = status.last_synced_at;
			const failedLast =
				status.last_error && (status.last_error_at ?? 0) > (status.last_synced_at ?? 0);
			// `running` is ignored: a sync this window didn't start has no completion
			// signal here, and showing "Syncing…" indefinitely would be wrong.
			if (failedLast && active === 0) {
				syncStatus.state = 'error';
				syncStatus.error = status.last_error ?? '';
			}
		})
		.catch(() => {
			// Outside Tauri — keep the local fallback.
		});
}

export async function trackSteamSync<T>(work: Promise<T>): Promise<T> {
	active += 1;
	syncStatus.state = 'syncing';
	try {
		const result = await work;
		const now = Math.floor(Date.now() / 1000);
		syncStatus.lastSyncedAt = now;
		syncStatus.error = '';
		try {
			localStorage.setItem(LAST_SYNC_KEY, String(now));
		} catch {
			// Per-device convenience only.
		}
		return result;
	} catch (error) {
		syncStatus.error = error instanceof Error ? error.message : String(error);
		throw error;
	} finally {
		active -= 1;
		if (active === 0) {
			syncStatus.state = syncStatus.error ? 'error' : 'idle';
		}
	}
}

/* The shell owns the actual sync (settings, toasts, page refresh). Pages ask it
   to run one — e.g. the "Sync now" button in an empty Games library. */
let syncHandler: (() => Promise<void> | void) | null = null;

export function registerSyncHandler(handler: () => Promise<void> | void) {
	syncHandler = handler;
	return () => {
		if (syncHandler === handler) syncHandler = null;
	};
}

export function requestSteamSync() {
	void syncHandler?.();
}
