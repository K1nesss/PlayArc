import { invoke } from '@tauri-apps/api/core';

// NOTE: Tauri v2 maps command arguments from camelCase keys (`gameId` → `game_id`).
// snake_case keys are silently ignored for optional arguments, so always use camelCase.
import { trackSteamSync } from '$lib/shell/sync-status.svelte';

export type GameDetailSession = {
	session_id: number;
	start_time: number;
	end_time: number | null;
	duration_seconds: number;
	formatted: string;
};

export type GameDetail = {
	id: number;
	appid: number | null;
	name: string;
	install_path: string | null;
	cover_path: string | null;
	hero_path: string | null;
	is_installed: boolean;
	is_hidden: boolean;
	created_at: number;
	updated_at: number;
	total_playtime_seconds: number;
	total_playtime_formatted: string;
	last_played_at: number | null;
	session_count: number;
	average_session_seconds: number;
	average_session_formatted: string;
	recent_sessions: GameDetailSession[];
	/** '#rrggbb' from the artwork, for the Hero tint. */
	accent_color: string | null;
};

export type IgnoredGameItem = {
	appid: number;
	name: string;
	ignored_at: number;
	reason: string | null;
};

export type HiddenGameItem = {
	id: number;
	appid: number | null;
	name: string;
	updated_at: number;
};

export type CreateManualGameRequest = {
	name: string;
	executable_path: string;
	icon_path?: string | null;
	cover_path?: string | null;
	hero_path?: string | null;
};

export type SteamIconSyncSummary = {
	matched_games: number;
	downloaded_icons: number;
	downloaded_covers: number;
	downloaded_heroes: number;
	updated_games: number;
	skipped_reason?: string | null;
};

export async function hello() {
	return await invoke<string>('hello');
}

export async function printDatabaseTables() {
	await invoke('print_database_tables');
}

export async function exportDatabase() {
	return await invoke<string>('export_database');
}

export async function importDatabase(base64Data: string) {
	await invoke('import_database', {
		base64Data: base64Data
	});
}

export async function openAppLogFile() {
	return await invoke<string>('open_app_log_file');
}

export async function getGameDetail(id: number, recentLimit = 20) {
	return await invoke<GameDetail>('game_detail', {
		id,
		recentLimit: recentLimit
	});
}

export async function createManualGame(request: CreateManualGameRequest) {
	return await invoke<number>('create_manual_game', { request });
}

export async function hideGame(gameId: number) {
	await invoke('hide_game', { gameId });
}

export async function deleteGame(gameId: number) {
	await invoke('delete_game', { gameId });
}

export async function deleteGameSession(sessionId: number) {
	await invoke('delete_game_session', { sessionId });
}

export async function getHiddenGamesList() {
	return await invoke<HiddenGameItem[]>('hidden_games_list');
}

export async function restoreHiddenGame(gameId: number) {
	await invoke('restore_hidden_game', { gameId });
}

export async function getIgnoredGamesList() {
	return await invoke<IgnoredGameItem[]>('ignored_games_list');
}

export async function restoreIgnoredGame(appId: number) {
	await invoke('restore_ignored_game', { appId });
}

export async function syncSteamOwnedGameIcons(
	apiKey: string,
	steam64Id: string,
	trigger = 'unknown'
) {
	return await trackSteamSync(
		invoke<SteamIconSyncSummary>('sync_steam_owned_game_icons', {
			apiKey,
			steam64Id,
			trigger
		})
	);
}

export async function syncAllSteamGameMedia(
	apiKey: string,
	steam64Id: string,
	trigger = 'unknown'
) {
	return await trackSteamSync(
		invoke<SteamIconSyncSummary>('sync_all_steam_game_media', {
			apiKey,
			steam64Id,
			trigger
		})
	);
}

/* ------------------------------------------------------------------ */
/* Redesign endpoints (Stage 4). Durations are seconds; no pre-       */
/* formatted strings — format with $lib/format.                       */
/* ------------------------------------------------------------------ */

export type DayValue = { day: string; seconds: number };

export type AnalyticsOverview = {
	range: { days: number | null; start: number; end: number; day_count: number };
	game_id: number | null;
	summary: {
		total_seconds: number;
		daily_average_seconds: number;
		session_count: number;
		average_session_seconds: number;
		median_session_seconds: number;
		longest_session_seconds: number;
		active_days: number;
		first_played_at: number | null;
		last_played_at: number | null;
	};
	daily: DayValue[];
	/** 7 rows (Monday-first) × 24 hours, seconds. */
	weekday_hour: number[][];
	/** Buckets: <30m, 30–60m, 1–2h, 2–3h, 3h+ (max_seconds null = open). */
	session_histogram: { min_seconds: number; max_seconds: number | null; sessions: number }[];
	top_games: {
		game_id: number;
		name: string;
		seconds: number;
		sessions: number;
		share: number;
		last_played_at: number | null;
	}[];
	insights: {
		peak_window: { weekday: number; start_hour: number; end_hour: number; seconds: number } | null;
		most_active_weekday: number | null;
		longest_streak_days: number;
		current_streak_days: number;
	};
	/** Last 53 weeks, independent of the range. */
	calendar: DayValue[];
};

/** `days` omitted/null = all time. `gameId` filters everything to one game. */
export async function getAnalyticsOverview(days: number | null, gameId?: number, topLimit = 10) {
	return await invoke<AnalyticsOverview>('analytics_overview', {
		days: days ?? null,
		gameId: gameId ?? null,
		topLimit: topLimit
	});
}

export type HomeGame = {
	game_id: number;
	name: string;
	cover_path: string | null;
	hero_path: string | null;
	icon_path: string | null;
	accent_color: string | null;
};

export type HomeSnapshot = {
	now: number;
	now_playing: {
		session_id: number;
		game: HomeGame;
		start_time: number;
		elapsed_seconds: number;
	} | null;
	today: { seconds: number; sessions: number };
	last_session: {
		session_id: number;
		game: HomeGame;
		start_time: number;
		end_time: number;
		duration_seconds: number;
	} | null;
};

export async function getHomeSnapshot() {
	return await invoke<HomeSnapshot>('home_snapshot');
}

export type LibrarySort = 'recent' | 'playtime' | 'name' | 'sessions';

export type LibraryGame = {
	id: number;
	appid: number | null;
	name: string;
	cover_path: string | null;
	icon_path: string | null;
	hero_path: string | null;
	accent_color: string | null;
	is_installed: boolean;
	total_playtime_seconds: number;
	last_played_at: number | null;
	session_count: number;
};

export async function getGamesLibrary(
	options: { search?: string; installedOnly?: boolean; sort?: LibrarySort } = {}
) {
	return await invoke<{ total: number; items: LibraryGame[] }>('games_library', {
		search: options.search?.trim() || null,
		installedOnly: options.installedOnly ?? null,
		sort: options.sort ?? 'recent'
	});
}

export type TimelineEntry = {
	session_id: number;
	game_id: number;
	appid: number | null;
	game_name: string;
	icon_path: string | null;
	cover_path: string | null;
	start_time: number;
	end_time: number | null;
	duration_seconds: number;
	is_active: boolean;
};

export type TimelineDay = {
	day: string;
	day_start: number;
	total_seconds: number;
	session_count: number;
	sessions: TimelineEntry[];
};

export type TimelineMonthPage = {
	month: string;
	total_seconds: number;
	session_count: number;
	days: TimelineDay[];
};

export async function getTimelineMonths(filters: { gameId?: number; query?: string } = {}) {
	return await invoke<{ month: string; sessions: number; seconds: number }[]>('timeline_months', {
		gameId: filters.gameId ?? null,
		query: filters.query?.trim() || null
	});
}

/** `month` = "YYYY-MM". */
export async function getTimelineDays(
	month: string,
	filters: { gameId?: number; query?: string } = {}
) {
	return await invoke<TimelineMonthPage>('timeline_days', {
		month,
		gameId: filters.gameId ?? null,
		query: filters.query?.trim() || null
	});
}

export type SteamSyncStatus = {
	last_synced_at: number | null;
	last_error: string | null;
	last_error_at: number | null;
	running: boolean;
};

export async function getSteamSyncStatus() {
	return await invoke<SteamSyncStatus>('steam_sync_status');
}

/** Delete all finished sessions (the library and a running session are kept). Returns count. */
export async function clearPlayHistory() {
	return await invoke<number>('clear_play_history');
}
