<!--
	Analysis (DOIT.md §7) — the statistics hub. "What patterns exist in my gaming?"
	Range: 7 / 30 / 90 days · 1 year · All.  Optional single-game filter.
-->
<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { RefreshCw } from '@lucide/svelte';
	import { getAnalyticsOverview, getGamesLibrary, type AnalyticsOverview } from '$lib/api';
	import { ANALYTICS_RANGES, loadSettingsFromStorage } from '$lib/settings';
	import { formatNumber, formatPlaytime, formatPlaytimeParts } from '$lib/format';
	import { locale, t } from '$lib/i18n.svelte';
	import {
		Button,
		DataPanel,
		InlineNotice,
		SegmentedControl,
		Select,
		Stat,
		StatGroup
	} from '$lib/components/ui';
	import DailyBarChart from '$lib/components/charts/DailyBarChart.svelte';
	import RankingChart from '$lib/components/charts/RankingChart.svelte';
	import ActivityHeatmap from '$lib/components/charts/ActivityHeatmap.svelte';
	import WeekdayHourHeatmap from '$lib/components/charts/WeekdayHourHeatmap.svelte';
	import SessionHistogram from '$lib/components/charts/SessionHistogram.svelte';

	type Range = (typeof ANALYTICS_RANGES)[number];
	const ALL_GAMES = 'all';
	const CACHE_PREFIX = 'velaris_analysis_v2';

	let range = $state<Range>(30);
	let gameFilter = $state<string>(ALL_GAMES);
	let gameOptions = $state<{ value: string; label: string }[]>([]);
	let data = $state<AnalyticsOverview | null>(null);
	let loading = $state(true);
	let refreshing = $state(false);
	let errorMessage = $state('');
	let requestId = 0;

	const rangeOptions = $derived(
		ANALYTICS_RANGES.map((days) => ({
			value: days,
			label:
				days === 0
					? t('全部', 'All')
					: days === 365
						? t('1 年', '1 year')
						: t(`${days} 天`, `${days} days`)
		}))
	);
	const gameId = $derived(gameFilter === ALL_GAMES ? undefined : Number(gameFilter));
	const summary = $derived(data?.summary);
	const total = $derived(formatPlaytimeParts(summary?.total_seconds ?? 0));
	const showAverage = $derived((data?.daily.length ?? 0) >= 7);
	const failed = $derived(Boolean(errorMessage) && !data);

	const weekdayNames = $derived.by(() => {
		const format = new Intl.DateTimeFormat(locale.current, { weekday: 'long' });
		return Array.from({ length: 7 }, (_, i) => format.format(new Date(2024, 0, 1 + i)));
	});
	const pad = (n: number) => String(n).padStart(2, '0');

	const patterns = $derived.by(() => {
		if (!data) return [];
		const { insights } = data;
		const peak = insights.peak_window;
		const days = (n: number) => t(`${n} 天`, `${n} ${n === 1 ? 'day' : 'days'}`);
		return [
			{
				label: t('高峰时段', 'Peak time'),
				value: peak
					? `${weekdayNames[peak.weekday]} · ${pad(peak.start_hour)}:00–${pad(peak.end_hour % 24)}:00`
					: '—'
			},
			{
				label: t('最活跃的一天', 'Most active day'),
				value:
					insights.most_active_weekday !== null ? weekdayNames[insights.most_active_weekday] : '—'
			},
			{
				label: t('会话中位时长', 'Median session'),
				value: data.summary.session_count
					? formatPlaytime(data.summary.median_session_seconds)
					: '—'
			},
			{ label: t('最长连续', 'Longest streak'), value: days(insights.longest_streak_days) },
			{ label: t('当前连续', 'Current streak'), value: days(insights.current_streak_days) }
		];
	});

	const rangeDescription = $derived(
		range === 0
			? t('全部记录', 'All time')
			: range === 365
				? t('过去一年', 'Past year')
				: t(`过去 ${range} 天`, `Past ${range} days`)
	);

	function cacheKey() {
		return `${CACHE_PREFIX}:${range}:${gameFilter}`;
	}

	async function load() {
		const id = ++requestId;
		const key = cacheKey();
		try {
			const cached = sessionStorage.getItem(key);
			if (cached) {
				data = JSON.parse(cached) as AnalyticsOverview;
				loading = false;
			}
		} catch {
			// Cache is a convenience.
		}
		if (data) refreshing = true;
		else loading = true;
		errorMessage = '';

		try {
			const next = await getAnalyticsOverview(range === 0 ? null : range, gameId, 10);
			if (id !== requestId) return;
			data = next;
			try {
				sessionStorage.setItem(key, JSON.stringify(next));
			} catch {
				// Ignore quota errors.
			}
		} catch (error) {
			if (id !== requestId) return;
			errorMessage = error instanceof Error ? error.message : String(error);
		} finally {
			if (id === requestId) {
				loading = false;
				refreshing = false;
			}
		}
	}

	function openGame(id: number) {
		void goto(resolve('/games/[id]', { id: String(id) }));
	}

	onMount(() => {
		void (async () => {
			const settings = await loadSettingsFromStorage();
			if ((ANALYTICS_RANGES as readonly number[]).includes(settings.analyticsDefaultRange)) {
				range = settings.analyticsDefaultRange as Range;
			}
			await load();
		})();
		getGamesLibrary({ sort: 'playtime' })
			.then((library) => {
				gameOptions = library.items
					.filter((game) => game.session_count > 0)
					.map((game) => ({ value: String(game.id), label: game.name }));
			})
			.catch(() => {});
	});
</script>

<div class="app-page h-full overflow-y-auto">
	<div class="app-page-inner analysis" aria-busy={loading || refreshing}>
		<header class="analysis-header">
			<div class="title-row">
				<h1 class="page-title">{t('分析', 'Analysis')}</h1>
				{#if refreshing}<span class="updating">{t('更新中…', 'Updating…')}</span>{/if}
			</div>
			<div class="toolbar-row">
				<SegmentedControl
					bind:value={range}
					options={rangeOptions}
					ariaLabel={t('时间范围', 'Time range')}
					onchange={() => void load()}
				/>
				<Select
					bind:value={gameFilter}
					options={[{ value: ALL_GAMES, label: t('全部游戏', 'All games') }, ...gameOptions]}
					ariaLabel={t('游戏', 'Game')}
					variant="ghost"
					onchange={() => void load()}
				/>
			</div>
		</header>

		{#if errorMessage}
			<InlineNotice tone="danger" title={t('无法加载分析数据', 'Couldn’t load analysis')}>
				{errorMessage}
				{#snippet action()}
					<Button size="sm" icon={RefreshCw} onclick={load}>{t('重试', 'Retry')}</Button>
				{/snippet}
			</InlineNotice>
		{/if}

		<!-- §7 Overview: one cohesive block -->
		<StatGroup boxed>
			<Stat label={t('总时长', 'Total time')} value={total.value} unit={total.unit} {loading} />
			<Stat
				label={t('日均', 'Daily avg')}
				value={formatPlaytime(summary?.daily_average_seconds ?? 0)}
				{loading}
			/>
			<Stat
				label={t('平均会话', 'Avg session')}
				value={formatPlaytime(summary?.average_session_seconds ?? 0)}
				hint={summary
					? t(
							`${formatNumber(summary.session_count)} 次会话`,
							`${formatNumber(summary.session_count)} sessions`
						)
					: undefined}
				{loading}
			/>
			<Stat
				label={t('活跃天数', 'Active days')}
				value={summary?.active_days ?? 0}
				hint={data ? t(`共 ${data.range.day_count} 天`, `of ${data.range.day_count}`) : undefined}
				{loading}
			/>
		</StatGroup>

		<DailyBarChart
			items={data?.daily ?? []}
			title={t('游玩趋势', 'Playtime trend')}
			description={showAverage
				? t('柱：每日时长 · 线：7 日移动平均', 'Bars: daily total · Line: 7-day moving average')
				: rangeDescription}
			{showAverage}
			height="lg"
			{loading}
			error={failed}
			onRetry={load}
		/>

		<div class="split">
			<WeekdayHourHeatmap
				grid={data?.weekday_hour ?? []}
				description={rangeDescription}
				{loading}
				error={failed}
				onRetry={load}
			/>
			<DataPanel
				title={t('规律', 'Patterns')}
				description={t('根据你的游玩记录计算', 'Computed from your play history')}
				height="auto"
				state={loading ? 'loading' : failed ? 'error' : summary?.session_count ? 'ready' : 'empty'}
				skeleton="rows"
				onRetry={load}
			>
				<dl class="patterns">
					{#each patterns as item (item.label)}
						<div>
							<dt>{item.label}</dt>
							<dd>{item.value}</dd>
						</div>
					{/each}
				</dl>
			</DataPanel>
		</div>

		<div class="split split--even">
			<SessionHistogram
				buckets={data?.session_histogram ?? []}
				{loading}
				error={failed}
				onRetry={load}
			/>
			{#if gameId === undefined}
				<RankingChart
					items={(data?.top_games ?? []).map((game) => ({
						id: game.game_id,
						name: game.name,
						seconds: game.seconds,
						sessions: game.sessions
					}))}
					description={t('时长前 10 · 点击查看详情', 'Top 10 by playtime · click to open')}
					{loading}
					error={failed}
					onRetry={load}
					onSelect={openGame}
				/>
			{:else}
				<DataPanel title={t('游戏详情', 'Game details')} height="sm">
					<div class="single-game">
						<p>{t('正在查看单个游戏的统计。', 'You’re looking at one game.')}</p>
						<Button size="sm" onclick={() => gameId !== undefined && openGame(gameId)}>
							{t('打开游戏页', 'Open game page')}
						</Button>
					</div>
				</DataPanel>
			{/if}
		</div>

		<ActivityHeatmap days={data?.calendar ?? []} {loading} error={failed} onRetry={load} />
	</div>
</div>

<style>
	.analysis {
		display: flex;
		flex-direction: column;
		gap: var(--space-4);
		padding-bottom: var(--space-8);
	}
	.analysis-header {
		display: flex;
		flex-direction: column;
		gap: var(--space-4);
		margin-bottom: var(--space-1);
	}
	.title-row {
		display: flex;
		align-items: baseline;
		gap: var(--space-3);
	}
	.updating {
		font-size: var(--text-xs);
		color: var(--text-tertiary);
	}
	.toolbar-row {
		display: flex;
		flex-wrap: wrap;
		align-items: center;
		gap: var(--space-3);
	}

	.split {
		display: grid;
		grid-template-columns: minmax(0, 2fr) minmax(240px, 1fr);
		gap: var(--space-4);
	}
	.split--even {
		grid-template-columns: repeat(2, minmax(0, 1fr));
	}
	@media (max-width: 1100px) {
		.split,
		.split--even {
			grid-template-columns: minmax(0, 1fr);
		}
	}

	.patterns {
		display: flex;
		flex-direction: column;
	}
	.patterns div {
		display: flex;
		align-items: baseline;
		justify-content: space-between;
		gap: var(--space-3);
		padding: 10px 0;
	}
	.patterns div + div {
		border-top: 1px solid var(--border-subtle);
	}
	.patterns dt {
		font-size: var(--text-sm);
		color: var(--text-secondary);
		white-space: nowrap;
	}
	.patterns dd {
		font-size: var(--text-base);
		font-weight: 600;
		text-align: right;
		color: var(--text-primary);
	}

	.single-game {
		display: flex;
		flex-direction: column;
		align-items: flex-start;
		justify-content: center;
		gap: var(--space-3);
		height: 100%;
		font-size: var(--text-sm);
		color: var(--text-secondary);
	}
</style>
