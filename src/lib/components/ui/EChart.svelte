<!--
	ECharts host. Owns lazy loading, init/dispose, resize, and re-theming, so chart
	components only describe their option.

	<EChart build={(tk) => ({ ...baseOption(tk), series: … })} ariaLabel="Daily playtime" />

	`build` is re-run whenever the reactive values it reads change, and whenever the
	theme / animation preference changes (observed on <html>'s class list).
-->
<script lang="ts">
	import type { EChartsType } from 'echarts/core';
	import type { EChartsOption } from 'echarts';
	import { onMount } from 'svelte';
	import { loadEcharts, createDebouncedCallback } from '$lib/charting';
	import { readChartTokens, type ChartTokens } from '$lib/charts/theme';

	interface Props {
		build: (tokens: ChartTokens) => EChartsOption;
		ariaLabel: string;
		/** CSS height; defaults to filling the DataPanel body. */
		height?: string;
		onclick?: (params: { dataIndex: number; name: string }) => void;
	}

	let { build, ariaLabel, height = '100%', onclick }: Props = $props();

	let container = $state<HTMLDivElement | null>(null);
	let tokens = $state<ChartTokens | null>(null);
	let visible = $state(false);
	let chart: EChartsType | null = null;
	let echartsLib: Awaited<ReturnType<typeof loadEcharts>> | null = null;

	// Rebuild whenever tokens or anything `build` reads changes.
	const option = $derived(tokens ? build(tokens) : null);

	$effect(() => {
		if (!option || !visible || !container) return;
		const next = option;
		void (async () => {
			echartsLib ??= await loadEcharts();
			if (!container) return;
			if (!chart) {
				chart = echartsLib.init(container, undefined, { renderer: 'canvas' });
				chart.on('click', (params) => {
					onclick?.({ dataIndex: params.dataIndex ?? -1, name: String(params.name ?? '') });
				});
			}
			chart.setOption(next, { notMerge: true });
		})();
	});

	onMount(() => {
		tokens = readChartTokens();

		const refreshTokens = () => {
			tokens = readChartTokens();
		};
		const classObserver = new MutationObserver(refreshTokens);
		classObserver.observe(document.documentElement, {
			attributes: true,
			attributeFilter: ['class']
		});

		const resize = createDebouncedCallback(() => chart?.resize(), 80);
		const resizeObserver = new ResizeObserver(resize);
		if (container) resizeObserver.observe(container);

		let intersection: IntersectionObserver | null = null;
		if ('IntersectionObserver' in window && container) {
			intersection = new IntersectionObserver(
				(entries) => {
					if (entries.some((entry) => entry.isIntersecting)) {
						visible = true;
						intersection?.disconnect();
					}
				},
				{ rootMargin: '200px 0px' }
			);
			intersection.observe(container);
		} else {
			visible = true;
		}

		return () => {
			classObserver.disconnect();
			resizeObserver.disconnect();
			intersection?.disconnect();
			resize.cancel();
			chart?.dispose();
			chart = null;
		};
	});
</script>

<div bind:this={container} class="ui-echart" style:height role="img" aria-label={ariaLabel}></div>

<style>
	.ui-echart {
		width: 100%;
		min-height: 0;
	}
</style>
