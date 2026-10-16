<!--
	Weekday × Hour heatmap (DOIT.md §7, §5, §24) — "At what times am I most likely
	to play?". 7 rows (Monday first) × 24 hours. One accent at six alpha steps,
	relative to the busiest hour in view (unlike the calendar, absolute amounts
	here depend on the range length, so relative intensity is what reads).
-->
<script lang="ts">
	import DataPanel from '$lib/components/ui/DataPanel.svelte';
	import { heatLevel, tooltipRow, tooltipTitle } from '$lib/charts/theme';
	import { formatPlaytime } from '$lib/format';
	import { locale, t } from '$lib/i18n.svelte';

	interface Props {
		/** 7 × 24 seconds, Monday-first (from analytics_overview.weekday_hour). */
		grid: number[][];
		title?: string;
		description?: string;
		loading?: boolean;
		error?: boolean;
		onRetry?: () => void;
	}

	let { grid, title, description, loading = false, error = false, onRetry }: Props = $props();

	const HOUR_LABELS = [0, 3, 6, 9, 12, 15, 18, 21];

	const max = $derived(Math.max(0, ...grid.flat()));
	const weekdays = $derived.by(() => {
		void locale.current;
		// 2024-01-01 was a Monday.
		const format = new Intl.DateTimeFormat(locale.current, { weekday: 'short' });
		const long = new Intl.DateTimeFormat(locale.current, { weekday: 'long' });
		return Array.from({ length: 7 }, (_, i) => ({
			short: format.format(new Date(2024, 0, 1 + i)),
			long: long.format(new Date(2024, 0, 1 + i))
		}));
	});

	const panelState = $derived(loading ? 'loading' : error ? 'error' : max <= 0 ? 'empty' : 'ready');

	let hover = $state<{ day: number; hour: number; x: number; y: number } | null>(null);
	let shell = $state<HTMLElement | null>(null);

	function show(event: PointerEvent, day: number, hour: number) {
		if (!shell) return;
		const cell = (event.currentTarget as HTMLElement).getBoundingClientRect();
		const host = shell.getBoundingClientRect();
		hover = { day, hour, x: cell.left - host.left + cell.width / 2, y: cell.top - host.top };
	}

	const pad = (n: number) => String(n).padStart(2, '0');
</script>

<DataPanel
	title={title ?? t('时段分布', 'When you play')}
	description={description ?? t('按星期和小时统计的游玩时长', 'Playtime by weekday and hour')}
	height="auto"
	state={panelState}
	{onRetry}
	emptyTitle={t('这段时间没有游玩记录', 'No playtime in this range')}
>
	<div
		bind:this={shell}
		class="heat"
		role="img"
		aria-label={t('星期 × 小时游玩热力图', 'Weekday by hour playtime heatmap')}
		onpointerleave={() => (hover = null)}
	>
		<div class="hours" aria-hidden="true">
			{#each Array.from({ length: 24 }, (_, h) => h) as h (h)}
				<span>{HOUR_LABELS.includes(h) ? pad(h) : ''}</span>
			{/each}
		</div>
		{#each grid as row, day (day)}
			<div class="row">
				<span class="weekday" aria-hidden="true">{weekdays[day]?.short}</span>
				<div class="cells">
					{#each row as seconds, hour (hour)}
						<span
							class="cell"
							style:background={`var(--heat-${heatLevel(seconds, max)})`}
							onpointerenter={(event) => show(event, day, hour)}
							role="presentation"
						></span>
					{/each}
				</div>
			</div>
		{/each}

		{#if hover}
			<div class="tip" style:left={`${hover.x}px`} style:top={`${hover.y}px`}>
				<!-- eslint-disable-next-line svelte/no-at-html-tags -->
				{@html tooltipTitle(
					`${weekdays[hover.day].long} · ${pad(hover.hour)}:00–${pad((hover.hour + 1) % 24)}:00`
				) +
					tooltipRow(
						t('游玩', 'Played'),
						grid[hover.day][hover.hour] > 0 ? formatPlaytime(grid[hover.day][hover.hour]) : '—'
					)}
			</div>
		{/if}

		<div class="legend" aria-hidden="true">
			<span>{t('少', 'Less')}</span>
			{#each [0, 1, 2, 3, 4, 5] as level (level)}
				<i style:background={`var(--heat-${level})`}></i>
			{/each}
			<span>{t('多', 'More')}</span>
		</div>
	</div>
</DataPanel>

<style>
	.heat {
		--label-w: 36px;
		position: relative;
		display: flex;
		flex-direction: column;
		gap: 3px;
	}
	.hours,
	.cells {
		display: grid;
		grid-template-columns: repeat(24, minmax(0, 1fr));
		gap: 3px;
	}
	.hours {
		margin-left: var(--label-w);
		font-size: 11px;
		color: var(--text-tertiary);
		font-variant-numeric: tabular-nums;
	}
	.row {
		display: flex;
		align-items: center;
	}
	.weekday {
		width: var(--label-w);
		flex-shrink: 0;
		font-size: 11px;
		color: var(--text-tertiary);
	}
	.cells {
		flex: 1;
	}
	.cell {
		height: 18px;
		border-radius: 3px;
		transition: box-shadow var(--duration-fast) var(--ease-standard);
	}
	.cell:hover {
		box-shadow: inset 0 0 0 1px var(--text-secondary);
	}
	.tip {
		position: absolute;
		z-index: var(--z-tooltip);
		transform: translate(-50%, calc(-100% - 8px));
		min-width: 160px;
		padding: 6px 10px;
		border: 1px solid var(--border-normal);
		border-radius: var(--radius-md);
		background: var(--bg-elevated);
		backdrop-filter: blur(16px);
		box-shadow: var(--elevation-2);
		font-size: var(--text-xs);
		color: var(--text-primary);
		white-space: nowrap;
		pointer-events: none;
	}
	.legend {
		display: flex;
		align-items: center;
		justify-content: flex-end;
		gap: 3px;
		margin-top: var(--space-2);
		font-size: 11px;
		color: var(--text-tertiary);
	}
	.legend i {
		width: 11px;
		height: 11px;
		border-radius: 2px;
	}
	.legend span:first-child {
		margin-right: 4px;
	}
	.legend span:last-child {
		margin-left: 4px;
	}
</style>
