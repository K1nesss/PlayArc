<!--
	Daily playtime (DOIT.md §7 "Playtime Trend", §23).
	Bars = raw daily playtime (accent). Optional line = 7-day moving average
	(neutral gray, 2px) — the smoothed trend.
-->
<script lang="ts">
	import type { EChartsOption } from 'echarts';
	import DataPanel from '$lib/components/ui/DataPanel.svelte';
	import EChart from '$lib/components/ui/EChart.svelte';
	import {
		barSeries,
		baseOption,
		categoryAxis,
		durationAxis,
		lineSeries,
		movingAverage,
		niceDurationScale,
		tooltipBase,
		tooltipRow,
		tooltipTitle,
		type ChartTokens
	} from '$lib/charts/theme';
	import { formatPlaytime, formatRelativeDay } from '$lib/format';
	import { locale, t } from '$lib/i18n.svelte';

	/** `day` = YYYY-MM-DD. Pass `label` instead for non-daily buckets (e.g. hours). */
	type Item = { day: string; label?: string; seconds: number };

	interface Props {
		items: Item[];
		title?: string;
		description?: string;
		showAverage?: boolean;
		height?: 'sm' | 'md' | 'lg';
		loading?: boolean;
		error?: boolean;
		onRetry?: () => void;
	}

	let {
		items,
		title,
		description,
		showAverage = false,
		height = 'md',
		loading = false,
		error = false,
		onRetry
	}: Props = $props();

	function toTimestamp(day: string) {
		const [y, m, d] = day.split('-').map(Number);
		return new Date(y, m - 1, d).getTime() / 1000;
	}

	function axisLabel(item: Item) {
		if (item.label) return item.label;
		const [, m, d] = item.day.split('-').map(Number);
		return `${m}/${d}`;
	}

	function tooltipLabel(item: Item) {
		return item.label ?? formatRelativeDay(toTimestamp(item.day));
	}

	const hasData = $derived(items.some((item) => item.seconds > 0));
	const panelState = $derived(loading ? 'loading' : error ? 'error' : !hasData ? 'empty' : 'ready');

	function build(tk: ChartTokens): EChartsOption {
		void locale.current;
		const values = items.map((item) => item.seconds);
		const average = movingAverage(values, 7);
		const labelAverage = t('7 日均值', '7-day avg');
		const labelPlayed = t('游玩', 'Played');

		return {
			...baseOption(tk),
			tooltip: {
				...tooltipBase(tk),
				trigger: 'axis',
				axisPointer: { type: 'shadow', shadowStyle: { color: tk.grid } },
				formatter: (params: unknown) => {
					const list = params as Array<{ dataIndex: number }>;
					const index = list[0]?.dataIndex ?? 0;
					const item = items[index];
					let html =
						tooltipTitle(tooltipLabel(item)) +
						tooltipRow(labelPlayed, formatPlaytime(item.seconds), tk.accent);
					if (showAverage) {
						html += tooltipRow(labelAverage, formatPlaytime(average[index]), tk.neutral);
					}
					return html;
				}
			},
			xAxis: categoryAxis(
				tk,
				items.map((item) => axisLabel(item))
			),
			yAxis: durationAxis(
				tk,
				niceDurationScale(Math.max(0, ...values, ...(showAverage ? average : [])))
			),
			series: [
				{ ...barSeries(tk, false, values.length), name: labelPlayed, data: values },
				...(showAverage
					? [{ ...lineSeries(tk.neutral), name: labelAverage, data: average, z: 3 }]
					: [])
			]
		};
	}
</script>

<DataPanel
	title={title ?? t('每日游玩', 'Daily playtime')}
	{description}
	{height}
	state={panelState}
	skeleton="bars"
	{onRetry}
	emptyTitle={t('这段时间没有游玩记录', 'No playtime in this range')}
>
	{#snippet actions()}
		{#if showAverage && panelState === 'ready'}
			<div class="legend" aria-hidden="true">
				<span><i class="swatch swatch--bar"></i>{t('每日', 'Daily')}</span>
				<span><i class="swatch swatch--line"></i>{t('7 日均值', '7-day avg')}</span>
			</div>
		{/if}
	{/snippet}
	<EChart {build} ariaLabel={title ?? t('每日游玩时长柱状图', 'Daily playtime bar chart')} />
</DataPanel>

<style>
	.legend {
		display: flex;
		gap: var(--space-3);
		font-size: var(--text-xs);
		color: var(--text-secondary);
	}
	.legend span {
		display: inline-flex;
		align-items: center;
		gap: 6px;
	}
	.swatch {
		display: inline-block;
	}
	.swatch--bar {
		width: 8px;
		height: 8px;
		border-radius: 2px;
		background: var(--chart-accent);
	}
	.swatch--line {
		width: 12px;
		height: 2px;
		border-radius: 1px;
		background: var(--chart-neutral);
	}
</style>
