<!--
	Session length distribution (DOIT.md §7) — "Are my sessions short and
	fragmented or long-form?". Buckets: <30m · 30–60m · 1–2h · 2–3h · 3h+.
	Single accent; count labelled on each bar.
-->
<script lang="ts">
	import type { EChartsOption } from 'echarts';
	import DataPanel from '$lib/components/ui/DataPanel.svelte';
	import EChart from '$lib/components/ui/EChart.svelte';
	import {
		barSeries,
		baseOption,
		categoryAxis,
		tooltipBase,
		tooltipRow,
		tooltipTitle,
		type ChartTokens
	} from '$lib/charts/theme';
	import { formatPlaytime } from '$lib/format';
	import { locale, t } from '$lib/i18n.svelte';

	type Bucket = { min_seconds: number; max_seconds: number | null; sessions: number };

	interface Props {
		buckets: Bucket[];
		loading?: boolean;
		error?: boolean;
		onRetry?: () => void;
	}

	let { buckets, loading = false, error = false, onRetry }: Props = $props();

	const total = $derived(buckets.reduce((sum, bucket) => sum + bucket.sessions, 0));
	const panelState = $derived(
		loading ? 'loading' : error ? 'error' : total === 0 ? 'empty' : 'ready'
	);

	/** "<30m", "30–60m", "1–2h", "2–3h", "3h+" */
	function label(bucket: Bucket) {
		const { min_seconds: min, max_seconds: max } = bucket;
		if (min === 0 && max) return `<${formatPlaytime(max)}`;
		if (!max) return `${formatPlaytime(min)}+`;
		if (max <= 3600) return `${min / 60}–${max / 60}m`;
		return `${min / 3600}–${max / 3600}h`;
	}

	function build(tk: ChartTokens): EChartsOption {
		void locale.current;
		const labels = buckets.map(label);
		return {
			...baseOption(tk),
			grid: { left: 4, right: 4, top: 20, bottom: 4, containLabel: true },
			tooltip: {
				...tooltipBase(tk),
				trigger: 'item',
				formatter: (params: unknown) => {
					const index = (params as { dataIndex: number }).dataIndex;
					const count = buckets[index].sessions;
					const share = total ? Math.round((count / total) * 100) : 0;
					return (
						tooltipTitle(labels[index]) +
						tooltipRow(t('会话', 'Sessions'), String(count)) +
						tooltipRow(t('占比', 'Share'), `${share}%`)
					);
				}
			},
			xAxis: categoryAxis(tk, labels),
			yAxis: { type: 'value', show: false, max: (value: { max: number }) => value.max * 1.15 },
			series: [
				{
					...barSeries(tk, false, 5),
					barMaxWidth: 40,
					data: buckets.map((bucket) => bucket.sessions),
					label: {
						show: true,
						position: 'top',
						color: tk.textSecondary,
						fontSize: 12,
						fontFamily: tk.fontFamily
					}
				}
			]
		};
	}
</script>

<DataPanel
	title={t('会话时长分布', 'Session length')}
	description={t('按单次会话时长统计', 'How long each session lasts')}
	height="sm"
	state={panelState}
	skeleton="bars"
	{onRetry}
	emptyTitle={t('这段时间没有会话', 'No sessions in this range')}
>
	<EChart {build} ariaLabel={t('会话时长分布柱状图', 'Session length histogram')} />
</DataPanel>
