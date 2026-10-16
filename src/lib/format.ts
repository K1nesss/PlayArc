/**
 * Shared display formatters (DOIT.md §36, §37).
 *
 * All durations are passed in SECONDS (the backend's unit). The frontend owns all
 * user-facing formatting; backend `*_formatted` fields are deprecated and will be
 * removed in Stage 4.
 *
 * Playtime rules (§37) — h/m units in every language:
 *   < 1 minute   → "<1m"
 *   < 1 hour     → "36m"
 *   ≥ 1 hour     → "3h 24m"
 *   headline     → "128.4h"     (one decimal, never raw floats)
 *   clock        → "01:42:36"   (live session timer)
 */
import type { AppLanguage } from '$lib/settings';
import { locale } from '$lib/i18n.svelte';

const SECONDS_PER_MINUTE = 60;
const SECONDS_PER_HOUR = 3600;
const SECONDS_PER_DAY = 86_400;

function lang(language?: AppLanguage): AppLanguage {
	return language ?? locale.current;
}

function clampSeconds(seconds: number | null | undefined) {
	const value = Number(seconds);
	return Number.isFinite(value) && value > 0 ? Math.floor(value) : 0;
}

/* ------------------------------------------------------------------ */
/* Playtime                                                           */
/* ------------------------------------------------------------------ */

/** Compact duration for lists, rows, tooltips: "36m", "3h 24m". */
export function formatPlaytime(seconds: number | null | undefined) {
	const total = clampSeconds(seconds);
	const hours = Math.floor(total / SECONDS_PER_HOUR);
	const minutes = Math.floor((total % SECONDS_PER_HOUR) / SECONDS_PER_MINUTE);

	if (hours === 0 && minutes === 0) {
		return total === 0 ? '0m' : '<1m';
	}
	if (hours === 0) {
		return `${minutes}m`;
	}
	if (minutes === 0) {
		return `${hours}h`;
	}
	return `${hours}h ${minutes}m`;
}

export type PlaytimeParts = { value: string; unit: 'h' | 'm' };

/**
 * Headline metric split into number + unit so the unit can be styled smaller.
 * ≥ 1h → hours with one decimal ("128.4" + "h"); < 1h → whole minutes ("36" + "m").
 * The language only affects digit grouping ("1,284.5h").
 */
export function formatPlaytimeParts(
	seconds: number | null | undefined,
	language?: AppLanguage
): PlaytimeParts {
	const total = clampSeconds(seconds);

	if (total < SECONDS_PER_HOUR) {
		return { value: String(Math.floor(total / SECONDS_PER_MINUTE)), unit: 'm' };
	}

	const hours = Math.round((total / SECONDS_PER_HOUR) * 10) / 10;
	return {
		value: formatNumber(hours, language, { minimumFractionDigits: 1, maximumFractionDigits: 1 }),
		unit: 'h'
	};
}

/** Headline metric as one string: "128.4h". */
export function formatPlaytimeHeadline(seconds: number | null | undefined, language?: AppLanguage) {
	const { value, unit } = formatPlaytimeParts(seconds, language);
	return `${value}${unit}`;
}

/** Live session clock: "01:42:36". */
export function formatClock(seconds: number | null | undefined) {
	const total = clampSeconds(seconds);
	const h = Math.floor(total / SECONDS_PER_HOUR);
	const m = Math.floor((total % SECONDS_PER_HOUR) / SECONDS_PER_MINUTE);
	const s = total % SECONDS_PER_MINUTE;
	return [h, m, s].map((part) => String(part).padStart(2, '0')).join(':');
}

/** Chart axis label: "2h", "1.5h", "30m", "0". Language-neutral on purpose (axes stay short). */
export function formatAxisDuration(seconds: number) {
	const value = Math.max(0, seconds);
	if (value === 0) {
		return '0';
	}
	if (value >= SECONDS_PER_HOUR) {
		const hours = value / SECONDS_PER_HOUR;
		return Number.isInteger(hours) ? `${hours}h` : `${hours.toFixed(1)}h`;
	}
	if (value >= SECONDS_PER_MINUTE) {
		return `${Math.round(value / SECONDS_PER_MINUTE)}m`;
	}
	return `${Math.round(value)}s`;
}

/* ------------------------------------------------------------------ */
/* Numbers                                                            */
/* ------------------------------------------------------------------ */

export function formatNumber(
	value: number,
	language?: AppLanguage,
	options?: Intl.NumberFormatOptions
) {
	return new Intl.NumberFormat(lang(language), options).format(value);
}

/* ------------------------------------------------------------------ */
/* Dates & times (inputs are Unix seconds)                            */
/* ------------------------------------------------------------------ */

function toDate(timestamp: number) {
	return new Date(timestamp * 1000);
}

function startOfDay(date: Date) {
	return new Date(date.getFullYear(), date.getMonth(), date.getDate());
}

/** Whole calendar days between two dates in local time (DST-safe). */
function calendarDayDiff(from: Date, to: Date) {
	return Math.round(
		(startOfDay(to).getTime() - startOfDay(from).getTime()) / (SECONDS_PER_DAY * 1000)
	);
}

/** "21:16" — always 24-hour. */
export function formatTime(timestamp: number, language?: AppLanguage) {
	return new Intl.DateTimeFormat(lang(language), {
		hour: '2-digit',
		minute: '2-digit',
		hourCycle: 'h23'
	}).format(toDate(timestamp));
}

/** "21:16 – 22:48", or "21:16 – Running" for an active session. */
export function formatTimeRange(
	start: number,
	end: number | null | undefined,
	language?: AppLanguage
) {
	const startText = formatTime(start, language);
	if (!end) {
		return `${startText} – ${lang(language) === 'zh-CN' ? '进行中' : 'Running'}`;
	}
	return `${startText} – ${formatTime(end, language)}`;
}

/**
 * Relative calendar day: "Today" / "Yesterday" / "Sep 28" (/ "Sep 28, 2025" in another year).
 * zh-CN: "今天" / "昨天" / "9月28日" (/ "2025年9月28日").
 */
export function formatRelativeDay(timestamp: number | null | undefined, language?: AppLanguage) {
	if (!timestamp) {
		return '—';
	}
	const l = lang(language);
	const date = toDate(timestamp);
	const now = new Date();
	const diff = calendarDayDiff(date, now);

	if (diff === 0) {
		return l === 'zh-CN' ? '今天' : 'Today';
	}
	if (diff === 1) {
		return l === 'zh-CN' ? '昨天' : 'Yesterday';
	}

	return new Intl.DateTimeFormat(l, {
		year: date.getFullYear() === now.getFullYear() ? undefined : 'numeric',
		month: l === 'zh-CN' ? 'long' : 'short',
		day: 'numeric'
	}).format(date);
}

/** Date header for grouped lists: "Today · Monday", "Sep 28 · Monday" / "9月28日 · 星期一". */
export function formatDayHeader(timestamp: number, language?: AppLanguage) {
	const l = lang(language);
	const weekday = new Intl.DateTimeFormat(l, { weekday: 'long' }).format(toDate(timestamp));
	return `${formatRelativeDay(timestamp, l)} · ${weekday}`;
}

/** Relative day + time: "Today · 22:14", "Sep 28 · 22:14". */
export function formatDayTime(timestamp: number | null | undefined, language?: AppLanguage) {
	if (!timestamp) {
		return '—';
	}
	return `${formatRelativeDay(timestamp, language)} · ${formatTime(timestamp, language)}`;
}

/** Absolute date-time for detail views: "2026/09/28 22:14" style, locale-aware. */
export function formatDateTime(timestamp: number | null | undefined, language?: AppLanguage) {
	if (!timestamp) {
		return '—';
	}
	return new Intl.DateTimeFormat(lang(language), {
		year: 'numeric',
		month: '2-digit',
		day: '2-digit',
		hour: '2-digit',
		minute: '2-digit',
		hourCycle: 'h23'
	}).format(toDate(timestamp));
}

/** Short month name for chart axes / heatmap columns: "Sep" / "9月". */
export function formatMonthShort(date: Date, language?: AppLanguage) {
	return new Intl.DateTimeFormat(lang(language), { month: 'short' }).format(date);
}

/** Full date: "Sep 28, 2026" / "2026年9月28日". */
export function formatFullDate(date: Date, language?: AppLanguage) {
	return new Intl.DateTimeFormat(lang(language), {
		year: 'numeric',
		month: 'short',
		day: 'numeric'
	}).format(date);
}

/**
 * Time since: "just now", "2 minutes ago", "3 days ago" / "刚刚", "2分钟前", "3天前".
 * Used for "last played" on game cards and the sync status.
 */
export function formatTimeAgo(timestamp: number | null | undefined, language?: AppLanguage) {
	if (!timestamp) {
		return '—';
	}
	const l = lang(language);
	const seconds = Math.round(Date.now() / 1000 - timestamp);

	if (seconds < 45) {
		return l === 'zh-CN' ? '刚刚' : 'just now';
	}

	const rtf = new Intl.RelativeTimeFormat(l, { numeric: 'auto' });
	if (seconds < SECONDS_PER_HOUR) {
		return rtf.format(-Math.max(1, Math.round(seconds / SECONDS_PER_MINUTE)), 'minute');
	}
	if (seconds < SECONDS_PER_DAY) {
		return rtf.format(-Math.round(seconds / SECONDS_PER_HOUR), 'hour');
	}

	const days = calendarDayDiff(toDate(timestamp), new Date());
	if (days < 30) {
		return rtf.format(-days, 'day');
	}
	return formatRelativeDay(timestamp, l);
}
