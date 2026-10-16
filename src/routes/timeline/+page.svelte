<!--
	Timeline (DOIT.md §6) — raw session history, not aggregated statistics.
	One month at a time, grouped by day under sticky date headers, dense rows.
	Day totals come from the backend for the whole day, so they're never partial.
-->
<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { page as appPage } from '$app/state';
	import { ChevronLeft, ChevronRight, History, RefreshCw, Search } from '@lucide/svelte';
	import {
		getGamesLibrary,
		getTimelineDays,
		getTimelineMonths,
		type TimelineEntry,
		type TimelineMonthPage
	} from '$lib/api';
	import { formatDayHeader, formatNumber, formatPlaytime, formatTimeRange } from '$lib/format';
	import { locale, t } from '$lib/i18n.svelte';
	import {
		Badge,
		Button,
		EmptyState,
		IconButton,
		InlineNotice,
		Input,
		Select,
		Skeleton
	} from '$lib/components/ui';
	import Cover from '$lib/components/game/Cover.svelte';

	type Month = { month: string; sessions: number; seconds: number };

	const SEARCH_DEBOUNCE_MS = 200;
	const ALL = 'all';

	let months = $state<Month[]>([]);
	let month = $state('');
	let page = $state<TimelineMonthPage | null>(null);
	let gameFilter = $state<string>(ALL);
	let gameOptions = $state<{ value: string; label: string }[]>([]);
	let search = $state('');
	let loading = $state(true);
	let pageLoading = $state(false);
	let errorMessage = $state('');
	let scroller = $state<HTMLElement | null>(null);
	let now = $state(Math.floor(Date.now() / 1000));
	let requestId = 0;

	const filters = $derived({
		gameId: gameFilter === ALL ? undefined : Number(gameFilter),
		query: search
	});
	const hasFilters = $derived(gameFilter !== ALL || search.trim() !== '');

	function monthLabel(key: string) {
		const [year, monthIndex] = key.split('-').map(Number);
		return new Intl.DateTimeFormat(locale.current, { year: 'numeric', month: 'short' }).format(
			new Date(year, monthIndex - 1, 1)
		);
	}

	function currentMonthKey() {
		const d = new Date();
		return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}`;
	}

	const monthOptions = $derived(
		months.map((item) => ({
			value: item.month,
			label: monthLabel(item.month),
			description: t(
				`${item.sessions} 次会话 · ${formatPlaytime(item.seconds)}`,
				`${item.sessions} sessions · ${formatPlaytime(item.seconds)}`
			)
		}))
	);
	const monthIndex = $derived(months.findIndex((item) => item.month === month));

	/** Reload the month list (filters changed), keeping the month if it still has data. */
	async function loadMonths() {
		const id = ++requestId;
		errorMessage = '';
		try {
			const next = await getTimelineMonths(filters);
			if (id !== requestId) return;
			months = next;
			if (!next.some((item) => item.month === month)) {
				month = next[0]?.month ?? currentMonthKey();
			}
			await loadPage(id);
		} catch (error) {
			if (id !== requestId) return;
			errorMessage = error instanceof Error ? error.message : String(error);
		} finally {
			if (id === requestId) loading = false;
		}
	}

	async function loadPage(id = ++requestId) {
		if (!months.length) {
			page = null;
			return;
		}
		pageLoading = true;
		try {
			const next = await getTimelineDays(month, filters);
			if (id !== requestId) return;
			page = next;
			scroller?.scrollTo({ top: 0 });
		} catch (error) {
			if (id !== requestId) return;
			errorMessage = error instanceof Error ? error.message : String(error);
		} finally {
			if (id === requestId) pageLoading = false;
		}
	}

	function stepMonth(delta: number) {
		// months are newest-first: "previous month" is the next index.
		const next = months[monthIndex + delta];
		if (next) {
			month = next.month;
			void loadPage();
		}
	}

	let searchTimer: number | null = null;
	function onSearchInput() {
		if (searchTimer !== null) window.clearTimeout(searchTimer);
		searchTimer = window.setTimeout(() => void loadMonths(), SEARCH_DEBOUNCE_MS);
	}

	function clearFilters() {
		gameFilter = ALL;
		search = '';
		void loadMonths();
	}

	function openGame(entry: TimelineEntry) {
		void goto(resolve('/games/[id]', { id: String(entry.game_id) }));
	}

	function liveDuration(entry: TimelineEntry) {
		return entry.is_active
			? Math.max(entry.duration_seconds, now - entry.start_time)
			: entry.duration_seconds;
	}

	onMount(() => {
		// "View in Timeline" from a game opens pre-filtered: /timeline?game=ID
		const gameParam = appPage.url.searchParams.get('game');
		if (gameParam && /^\d+$/.test(gameParam)) gameFilter = gameParam;
		void loadMonths();
		getGamesLibrary({ sort: 'name' })
			.then((library) => {
				gameOptions = library.items
					.filter((game) => game.session_count > 0)
					.map((game) => ({ value: String(game.id), label: game.name }));
			})
			.catch(() => {});
		const tick = window.setInterval(() => (now = Math.floor(Date.now() / 1000)), 30_000);
		return () => {
			window.clearInterval(tick);
			if (searchTimer !== null) window.clearTimeout(searchTimer);
		};
	});
</script>

<div bind:this={scroller} class="app-page h-full overflow-y-auto">
	<div class="app-page-inner timeline">
		<header class="timeline-header">
			<div class="title-row">
				<div>
					<h1 class="page-title">{t('时间线', 'Timeline')}</h1>
					<p class="page-subtitle tabular-nums">
						{#if page}
							{t(
								`${formatNumber(page.session_count)} 次会话 · ${formatPlaytime(page.total_seconds)}`,
								`${formatNumber(page.session_count)} sessions · ${formatPlaytime(page.total_seconds)}`
							)}
						{:else}&nbsp;{/if}
					</p>
				</div>
				{#if months.length}
					<div class="month-nav">
						<IconButton
							icon={ChevronLeft}
							label={t('上个月', 'Previous month')}
							size="sm"
							disabled={monthIndex >= months.length - 1}
							onclick={() => stepMonth(1)}
						/>
						<Select
							bind:value={month}
							options={monthOptions}
							ariaLabel={t('月份', 'Month')}
							variant="ghost"
							placement="bottom-end"
							onchange={() => void loadPage()}
						/>
						<IconButton
							icon={ChevronRight}
							label={t('下个月', 'Next month')}
							size="sm"
							disabled={monthIndex <= 0}
							onclick={() => stepMonth(-1)}
						/>
					</div>
				{/if}
			</div>
			<div class="toolbar-row">
				<Select
					bind:value={gameFilter}
					options={[{ value: ALL, label: t('全部游戏', 'All games') }, ...gameOptions]}
					ariaLabel={t('游戏', 'Game')}
					variant="ghost"
					onchange={() => void loadMonths()}
				/>
				<Input
					bind:value={search}
					icon={Search}
					clearable
					placeholder={t('搜索游戏名…', 'Search by game…')}
					aria-label={t('搜索', 'Search')}
					class="search"
					oninput={onSearchInput}
					onclear={() => void loadMonths()}
				/>
			</div>
		</header>

		{#if errorMessage}
			<InlineNotice tone="danger" title={t('无法加载时间线', 'Couldn’t load the timeline')}>
				{errorMessage}
				{#snippet action()}
					<Button size="sm" icon={RefreshCw} onclick={() => loadMonths()}
						>{t('重试', 'Retry')}</Button
					>
				{/snippet}
			</InlineNotice>
		{/if}

		{#if loading}
			{#each [3, 2] as rows, d (d)}
				<div class="day">
					<div class="day-header"><Skeleton width="160px" height="12px" /></div>
					<div class="day-list">
						{#each Array.from({ length: rows }, (_, i) => i) as i (i)}
							<div class="row-skeleton">
								<Skeleton width="32px" height="32px" radius="sm" />
								<div class="flex flex-1 flex-col gap-2">
									<Skeleton width="180px" height="12px" />
									<Skeleton width="96px" height="10px" />
								</div>
								<Skeleton width="48px" height="12px" />
							</div>
						{/each}
					</div>
				</div>
			{/each}
		{:else if !months.length && !errorMessage}
			{#if hasFilters}
				<EmptyState
					bordered
					icon={Search}
					title={t('没有匹配的会话', 'No matching sessions')}
					description={t('试试其他游戏或关键词。', 'Try another game or search term.')}
				>
					{#snippet action()}
						<Button size="sm" onclick={clearFilters}>{t('清除筛选', 'Clear filters')}</Button>
					{/snippet}
				</EmptyState>
			{:else}
				<EmptyState
					bordered
					icon={History}
					title={t('还没有游玩记录', 'No play history yet')}
					description={t(
						'游戏会话会在你游玩时自动记录在这里。',
						'Sessions appear here automatically as you play.'
					)}
				/>
			{/if}
		{:else if page}
			<div class="days" class:is-refreshing={pageLoading}>
				{#each page.days as day (day.day)}
					<section class="day" aria-labelledby={`day-${day.day}`}>
						<header class="day-header">
							<h2 id={`day-${day.day}`} class="day-title">{formatDayHeader(day.day_start)}</h2>
							<span class="day-total tabular-nums">
								{formatPlaytime(day.total_seconds)}
								<span aria-hidden="true">·</span>
								{t(
									`${day.session_count} 次会话`,
									`${day.session_count} ${day.session_count === 1 ? 'session' : 'sessions'}`
								)}
							</span>
						</header>
						<ul class="day-list">
							{#each day.sessions as entry (entry.session_id)}
								<li>
									<button type="button" class="row" onclick={() => openGame(entry)}>
										<Cover
											name={entry.game_name}
											path={entry.icon_path ?? entry.cover_path}
											shape="square"
											size={32}
											radius="sm"
										/>
										<span class="row-main">
											<span class="row-name">{entry.game_name}</span>
											<span class="row-range tabular-nums">
												{formatTimeRange(entry.start_time, entry.end_time)}
											</span>
										</span>
										{#if entry.is_active}<Badge tone="success">{t('进行中', 'Live')}</Badge>{/if}
										<span class="row-duration tabular-nums"
											>{formatPlaytime(liveDuration(entry))}</span
										>
									</button>
								</li>
							{/each}
						</ul>
					</section>
				{/each}
			</div>
		{/if}
	</div>
</div>

<style>
	.timeline {
		padding-bottom: var(--space-8);
	}
	.timeline-header {
		display: flex;
		flex-direction: column;
		gap: var(--space-4);
		margin-bottom: var(--space-4);
	}
	.title-row {
		display: flex;
		align-items: flex-start;
		justify-content: space-between;
		gap: var(--space-4);
	}
	.month-nav {
		display: flex;
		align-items: center;
		gap: 2px;
	}
	.toolbar-row {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: var(--space-2);
	}
	.toolbar-row :global(.search) {
		width: min(280px, 100%);
	}

	.days {
		transition: opacity var(--duration-fast) var(--ease-standard);
	}
	.days.is-refreshing {
		opacity: 0.6;
	}

	/* §6: sticky date headers keep the date in view while scrolling. */
	.day-header {
		position: sticky;
		/* Sticky offsets are measured inside the scroller's padding; cancel it so the
		   header pins flush to the top edge instead of 24px below it. */
		top: calc(-1 * var(--page-padding-y));
		z-index: var(--z-sticky);
		display: flex;
		align-items: baseline;
		justify-content: space-between;
		gap: var(--space-3);
		margin: 0 calc(-1 * var(--space-1));
		padding: var(--space-4) var(--space-1) var(--space-2);
		background: var(--bg-app);
	}
	.day-title {
		font-size: var(--text-sm);
		font-weight: 600;
		color: var(--text-primary);
	}
	.day-total {
		display: inline-flex;
		gap: 6px;
		font-size: var(--text-sm);
		color: var(--text-secondary);
	}

	.day-list {
		border: 1px solid var(--border-subtle);
		border-radius: var(--radius-xl);
		background: var(--bg-surface);
		box-shadow: var(--elevation-1);
		overflow: hidden;
	}
	.day-list li + li,
	.row-skeleton + .row-skeleton {
		border-top: 1px solid var(--border-subtle);
	}

	.row,
	.row-skeleton {
		display: flex;
		align-items: center;
		gap: var(--space-3);
		width: 100%;
		height: 56px;
		padding: 0 var(--space-4) 0 var(--space-3);
	}
	.row {
		text-align: left;
		color: inherit;
		transition: background-color var(--duration-fast) var(--ease-standard);
	}
	.row:hover {
		background: var(--state-hover);
	}
	.row:active {
		background: var(--state-active);
	}
	.row:focus-visible {
		outline-offset: -2px;
	}
	.row-main {
		display: flex;
		flex: 1;
		flex-direction: column;
		min-width: 0;
	}
	.row-name {
		overflow: hidden;
		text-overflow: ellipsis;
		white-space: nowrap;
		font-size: var(--text-base);
		font-weight: 500;
		color: var(--text-primary);
	}
	.row-range {
		font-size: var(--text-xs);
		color: var(--text-secondary);
	}
	.row-duration {
		min-width: 64px;
		text-align: right;
		font-size: var(--text-base);
		font-weight: 500;
		color: var(--text-primary);
	}
</style>
