<!--
	GitHub-style activity calendar (DOIT.md §7, §24).
	One accent hue at six alpha steps (--heat-0 … --heat-5). Fixed thresholds, so a
	color always means the same amount of play:
	  0 none · 1 <30m · 2 <1h · 3 <2h · 4 <4h · 5 ≥4h
	Derived values: active days, longest streak, current streak.
-->
<script lang="ts">
	import { onMount } from 'svelte';
	import type { DayValue } from '$lib/api';
	import { formatFullDate, formatMonthShort, formatPlaytime } from '$lib/format';
	import { locale, t } from '$lib/i18n.svelte';
	import DataPanel from '$lib/components/ui/DataPanel.svelte';
	import { tooltipRow, tooltipTitle } from '$lib/charts/theme';

	interface Props {
		days: DayValue[];
		loading?: boolean;
		error?: boolean;
		onRetry?: () => void;
		title?: string;
		description?: string;
	}

	let { days, loading = false, error = false, onRetry, title, description }: Props = $props();

	const CELL = 11;
	const GAP = 3;
	const STRIDE = CELL + GAP;
	const LABEL_W = 28;
	const MONTH_H = 18;
	const GRID_H = 7 * CELL + 6 * GAP;
	const THRESHOLDS = [30 * 60, 3600, 2 * 3600, 4 * 3600];

	type Cell = DayValue & { level: number; week: number; weekday: number; date: Date };

	function parseLocalDate(value: string) {
		const [y, m, d] = value.split('-').map(Number);
		return new Date(y, m - 1, d);
	}
	const mondayFirst = (date: Date) => (date.getDay() + 6) % 7;
	const dayDiff = (a: Date, b: Date) => Math.round((b.getTime() - a.getTime()) / 86_400_000);

	function levelFor(seconds: number) {
		if (seconds <= 0) return 0;
		const index = THRESHOLDS.findIndex((limit) => seconds < limit);
		return index === -1 ? 5 : index + 1;
	}

	function streaks(items: DayValue[]) {
		let longest = 0;
		let run = 0;
		for (const item of items) {
			run = item.seconds > 0 ? run + 1 : 0;
			longest = Math.max(longest, run);
		}
		// Current streak: count back from the last day; today may still be empty.
		let current = 0;
		for (let i = items.length - 1; i >= 0; i--) {
			if (items[i].seconds > 0) current += 1;
			else if (i === items.length - 1) continue;
			else break;
		}
		return { longest, current };
	}

	const calendar = $derived.by(() => {
		void locale.current;
		const sorted = [...days].sort((a, b) => a.day.localeCompare(b.day));
		if (sorted.length === 0) {
			return { cells: [] as Cell[], months: [], weeks: 53, active: 0, longest: 0, current: 0 };
		}
		const first = parseLocalDate(sorted[0].day);
		// Local scratch value inside a derived computation — not reactive state.
		// eslint-disable-next-line svelte/prefer-svelte-reactivity
		const start = new Date(first);
		start.setDate(first.getDate() - mondayFirst(first));

		const cells: Cell[] = sorted.map((item) => {
			const date = parseLocalDate(item.day);
			return {
				...item,
				date,
				level: levelFor(item.seconds),
				week: Math.floor(dayDiff(start, date) / 7),
				weekday: mondayFirst(date)
			};
		});

		const months: { label: string; week: number }[] = [];
		let previous = -1;
		for (const cell of cells) {
			if (cell.date.getMonth() !== previous && cell.date.getDate() <= 7) {
				months.push({ label: formatMonthShort(cell.date), week: cell.week });
			}
			previous = cell.date.getMonth();
		}

		const { longest, current } = streaks(sorted);
		return {
			cells,
			months,
			weeks: cells[cells.length - 1].week + 1,
			active: sorted.filter((item) => item.seconds > 0).length,
			longest,
			current
		};
	});

	const viewWidth = $derived(LABEL_W + calendar.weeks * STRIDE);
	const weekdayLabels = $derived(
		locale.current === 'zh-CN'
			? ['一', '', '三', '', '五', '', '']
			: ['Mon', '', 'Wed', '', 'Fri', '', '']
	);

	let shell = $state<HTMLDivElement | null>(null);
	let width = $state(0);
	let hover = $state<{ cell: Cell; x: number; y: number } | null>(null);

	onMount(() => {
		if (!shell) return;
		const observer = new ResizeObserver(() => (width = shell?.clientWidth ?? 0));
		observer.observe(shell);
		return () => observer.disconnect();
	});

	const visibleMonths = $derived(
		calendar.months.filter((_, index) => width >= 560 || index % 2 === 0)
	);

	function showCell(event: PointerEvent, cell: Cell) {
		if (!shell) return;
		const rect = (event.target as SVGRectElement).getBoundingClientRect();
		const host = shell.getBoundingClientRect();
		hover = { cell, x: rect.left - host.left + rect.width / 2, y: rect.top - host.top };
	}

	const panelState = $derived(
		loading ? 'loading' : error ? 'error' : days.length === 0 ? 'empty' : 'ready'
	);
</script>

<DataPanel
	title={title ?? t('活动热力图', 'Activity')}
	description={description ?? t('过去一年每天的游玩时长', 'Daily playtime over the past year')}
	height="auto"
	state={panelState}
	{onRetry}
	emptyTitle={t('还没有游玩记录', 'No play history yet')}
>
	{#snippet actions()}
		{#if panelState === 'ready'}
			<dl class="heat-stats">
				<div>
					<dt>{t('活跃天数', 'Active days')}</dt>
					<dd>{calendar.active}</dd>
				</div>
				<div>
					<dt>{t('最长连续', 'Longest streak')}</dt>
					<dd>{calendar.longest}</dd>
				</div>
				<div>
					<dt>{t('当前连续', 'Current streak')}</dt>
					<dd>{calendar.current}</dd>
				</div>
			</dl>
		{/if}
	{/snippet}

	<div
		bind:this={shell}
		class="heat-shell"
		onpointerleave={() => (hover = null)}
		role="presentation"
	>
		<svg
			viewBox={`0 0 ${viewWidth} ${MONTH_H + GRID_H}`}
			class="heat-svg"
			role="img"
			aria-label={t('过去一年的每日游玩热力图', 'Daily playtime heatmap for the past year')}
		>
			<g class="heat-label" aria-hidden="true">
				{#each visibleMonths as month (month.week)}
					<text x={LABEL_W + month.week * STRIDE} y="10">{month.label}</text>
				{/each}
				{#each weekdayLabels as label, index (index)}
					<text x={LABEL_W - 6} y={MONTH_H + index * STRIDE + CELL - 2} text-anchor="end">
						{label}
					</text>
				{/each}
			</g>
			<g>
				{#each calendar.cells as cell (cell.day)}
					<rect
						x={LABEL_W + cell.week * STRIDE}
						y={MONTH_H + cell.weekday * STRIDE}
						width={CELL}
						height={CELL}
						rx="2"
						class="heat-cell"
						style:fill={`var(--heat-${cell.level})`}
						onpointerenter={(event) => showCell(event, cell)}
						role="presentation"
					></rect>
				{/each}
			</g>
		</svg>

		{#if hover}
			<div class="heat-tooltip" style:left={`${hover.x}px`} style:top={`${hover.y}px`}>
				<!-- eslint-disable-next-line svelte/no-at-html-tags -->
				{@html tooltipTitle(formatFullDate(hover.cell.date)) +
					tooltipRow(
						t('游玩', 'Played'),
						hover.cell.seconds > 0 ? formatPlaytime(hover.cell.seconds) : '—'
					)}
			</div>
		{/if}

		<div class="heat-legend" aria-hidden="true">
			<span>{t('少', 'Less')}</span>
			{#each [0, 1, 2, 3, 4, 5] as level (level)}
				<span class="heat-swatch" style:background={`var(--heat-${level})`}></span>
			{/each}
			<span>{t('多', 'More')}</span>
		</div>
	</div>
</DataPanel>

<style>
	.heat-shell {
		position: relative;
	}
	.heat-svg {
		display: block;
		width: 100%;
		height: auto;
	}
	.heat-label text {
		fill: var(--text-tertiary);
		font-size: 10px;
	}
	.heat-cell {
		transition: opacity var(--duration-fast) var(--ease-standard);
	}
	.heat-cell:hover {
		stroke: var(--text-secondary);
		stroke-width: 1;
	}
	.heat-tooltip {
		position: absolute;
		z-index: var(--z-tooltip);
		transform: translate(-50%, calc(-100% - 8px));
		min-width: 140px;
		padding: 6px 10px;
		border: 1px solid var(--border-normal);
		border-radius: var(--radius-md);
		background: var(--bg-elevated);
		backdrop-filter: blur(16px);
		box-shadow: var(--elevation-2);
		font-size: var(--text-xs);
		color: var(--text-primary);
		pointer-events: none;
		white-space: nowrap;
	}
	.heat-legend {
		display: flex;
		align-items: center;
		justify-content: flex-end;
		gap: 3px;
		margin-top: var(--space-3);
		font-size: 11px;
		color: var(--text-tertiary);
	}
	.heat-legend span:first-child {
		margin-right: 4px;
	}
	.heat-legend span:last-child {
		margin-left: 4px;
	}
	.heat-swatch {
		width: 11px;
		height: 11px;
		border-radius: 2px;
	}
	.heat-stats {
		display: flex;
		gap: var(--space-5);
	}
	.heat-stats div {
		display: flex;
		flex-direction: column;
		align-items: flex-end;
	}
	.heat-stats dt {
		font-size: 11px;
		color: var(--text-tertiary);
	}
	.heat-stats dd {
		font-size: var(--text-base);
		font-weight: 600;
		font-variant-numeric: tabular-nums;
		color: var(--text-primary);
	}
</style>
