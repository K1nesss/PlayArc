<!--
	Game ranking (DOIT.md §7, §23, §24): horizontal bars, top 10 only, one accent
	color for every game — no per-game brand colors, no pies. Values are labelled at
	the bar end; full name + sessions in the tooltip.
-->
<script lang="ts" module>
	export type RankingItem = { id: number; name: string; seconds: number; sessions?: number };
</script>

<script lang="ts">
	import type { EChartsOption } from 'echarts';
	import DataPanel from '$lib/components/ui/DataPanel.svelte';
	import EChart from '$lib/components/ui/EChart.svelte';
	import {
		barSeries,
		baseOption,
		tooltipBase,
		tooltipRow,
		tooltipTitle,
		type ChartTokens
	} from '$lib/charts/theme';
	import { formatPlaytime, formatPlaytimeHeadline } from '$lib/format';
	import { locale, t } from '$lib/i18n.svelte';

	interface Props {
		items: RankingItem[];
		limit?: number;
		title?: string;
		description?: string;
		loading?: boolean;
		error?: boolean;
		onRetry?: () => void;
		onSelect?: (id: number) => void;
	}

	let {
		items,
		limit = 10,
		title,
		description,
		loading = false,
		error = false,
		onRetry,
		onSelect
	}: Props = $props();

	const ROW_HEIGHT = 28;

	const ranked = $derived(
		[...items]
			.filter((item) => item.seconds > 0)
			.sort((a, b) => b.seconds - a.seconds)
			.slice(0, limit)
	);
	const panelState = $derived(
		loading ? 'loading' : error ? 'error' : ranked.length === 0 ? 'empty' : 'ready'
	);
	const chartHeight = $derived(`${Math.max(3, ranked.length) * ROW_HEIGHT + 8}px`);

	function truncate(name: string, max = 18) {
		return name.length > max ? `${name.slice(0, max - 1)}…` : name;
	}

	function build(tk: ChartTokens): EChartsOption {
		void locale.current;
		// ECharts draws category axes bottom-up; reverse so #1 is on top.
		const rows = [...ranked].reverse();
		const sessionsLabel = t('会话', 'Sessions');
		return {
			...baseOption(tk),
			grid: { left: 4, right: 56, top: 0, bottom: 0, containLabel: true },
			tooltip: {
				...tooltipBase(tk),
				trigger: 'item',
				formatter: (params: unknown) => {
					const row = rows[(params as { dataIndex: number }).dataIndex];
					let html =
						tooltipTitle(row.name) + tooltipRow(t('时长', 'Time'), formatPlaytime(row.seconds));
					if (row.sessions !== undefined) html += tooltipRow(sessionsLabel, String(row.sessions));
					return html;
				}
			},
			xAxis: { type: 'value', show: false, max: (value: { max: number }) => value.max * 1.02 },
			yAxis: {
				type: 'category',
				data: rows.map((row) => truncate(row.name)),
				axisLine: { show: false },
				axisTick: { show: false },
				axisLabel: { color: tk.textSecondary, fontSize: 12, fontFamily: tk.fontFamily, margin: 12 }
			},
			series: [
				{
					...barSeries(tk, true),
					barCategoryGap: '40%',
					cursor: onSelect ? 'pointer' : 'default',
					data: rows.map((row) => row.seconds),
					label: {
						show: true,
						position: 'right',
						distance: 8,
						color: tk.textSecondary,
						fontSize: 12,
						fontFamily: tk.fontFamily,
						formatter: (params: { value: unknown }) => formatPlaytimeHeadline(Number(params.value))
					}
				}
			]
		};
	}
</script>

<DataPanel
	title={title ?? t('游戏排行', 'Top games')}
	{description}
	height="auto"
	state={panelState}
	skeleton="rows"
	{onRetry}
	emptyTitle={t('这段时间没有游玩记录', 'No playtime in this range')}
>
	<EChart
		{build}
		height={chartHeight}
		ariaLabel={t('游戏时长排行', 'Playtime ranking by game')}
		onclick={({ dataIndex }) => {
			const row = [...ranked].reverse()[dataIndex];
			if (row) onSelect?.(row.id);
		}}
	/>
</DataPanel>
